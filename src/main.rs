//! Secure P2P Messaging Tool
//!
//! This application encrypts and decrypts messages, photos (yet to be implemented), files (yet to be implemented) using either:
//! - AES-128 in GCM mode (modern authenticated encryption)
//! - DES in CBC mode (older 56 bit key support)
//!
//!  Includes password-based key derivation (PBKDF2) and an egui-based GUI frontend.


use aes_gcm::{Aes128Gcm, Key, Nonce}; // AES-128 GCM mode
use aes_gcm::aead::{Aead, KeyInit}; // Encryption trait interface
use sha2::Sha256; // SHA-256 hash function
use pbkdf2::pbkdf2; // Password based key derivation
use hmac::Hmac;
use rand_core::RngCore; // Secure random number gen
use rand_core::OsRng; // random num gen part 2
use base64::{engine::general_purpose, Engine as _}; // Allows for base 64 encoding and decoding

use des::Des; // DES cipher
use cbc::{Encryptor as CbcEncryptor, Decryptor as CbcDecryptor};
use cipher::{block_padding::Pkcs7, BlockEncryptMut, BlockDecryptMut, KeyIvInit};

use aes_gcm::aead::{Aead, KeyInit};
use std::fs;

use eframe::egui; // GUI things

const PBKDF2_ITERATIONS: u32 = 100_000;
const NONCE_LENGTH: usize = 12;

/// Enum to represent encryption types
enum Mode {
    AES,
    DES,
}

/// Derive a key from a password using PBKDF2-HMAC-SHA256
///
/// - For AES: returns an 16 byte (128-bit) key
/// - For DES: returns an 8 byte (56-bit + padding) key
///
/// # Arguments
/// * `password` - Shared secret string
/// * `mode` - Which cipher you're deriving a key for
fn derive_key(password: &str, mode:Mode) -> Vec<u8> {
    let salt = b"chat_salt"; // Currently a static salt value
    let mut key = vec![0u8; match mode {
        Mode::AES => 16,
        Mode::DES => 8,
    }];
    pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

/// Encrypt a message with AES-GCM using the shared password.
///
/// # Returns
/// A tuple of base64-encoded (ciphertext, nonce)
fn encrypt_aes(message: &str, password: &str) -> (String, String) {
    let derived = derive_key(password, Mode::AES);
    let key = Key::<Aes128Gcm>::from_slice(&derived);
    let cipher = Aes128Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    OsRng.fill_bytes(&mut nonce_bytes); // Generate a fresh nonce
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, message.as_bytes()).expect("encryption failure!");

    (
        general_purpose::STANDARD.encode(ciphertext),
        general_purpose::STANDARD.encode(nonce_bytes),
    )
}


/// Decrypts a base64-encoded message with AES-GCM and the shared password.
///
/// # Returns
/// The original plaintext as a UTF-8 string
fn decrypt_aes(ciphertext_b64: &str, nonce_b64: &str, password: &str) -> String {
    let key_bytes = derive_key(password, Mode::AES);
    let key = aes_gcm::Key::<Aes128Gcm>::from_slice(&key_bytes);
    let cipher = Aes128Gcm::new(key);

    let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64).unwrap();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = general_purpose::STANDARD.decode(ciphertext_b64).unwrap();

    let plaintext = cipher.decrypt(nonce, ciphertext.as_ref()).expect("decryption failure!");
    String::from_utf8(plaintext).expect("invalid utf8")
}

/// DES-CBC encryption
///
/// # Returns
/// A tuple of base64-encoded (ciphertext, key, iv)
fn encrypt_des(message: &str, password: &str) -> (String, String, String) {
    let key = derive_key(password, Mode::DES);
    let mut iv = [0u8; 8];
    OsRng.fill_bytes(&mut iv);

    let mut buffer = vec![0u8; message.len() + 8]; // Add padding buffer
    buffer[..message.len()].copy_from_slice(message.as_bytes());

    let cipher = CbcEncryptor::<Des>::new_from_slices(&key, &iv).unwrap();
    let ciphertext = cipher.encrypt_padded_mut::<Pkcs7>(&mut buffer, message.len()).unwrap();

    (
        general_purpose::STANDARD.encode(ciphertext),
        general_purpose::STANDARD.encode(&key),
        general_purpose::STANDARD.encode(&iv),
    )
}

/// DES-CBC decryption
///
/// # Returns
/// The decrypted plaintext
fn decrypt_des(ciphertext_b64: &str, key_b64: &str, iv_b64: &str) -> String {
    let key = general_purpose::STANDARD.decode(key_b64).unwrap();
    let iv = general_purpose::STANDARD.decode(iv_b64).unwrap();
    let mut buffer = general_purpose::STANDARD.decode(ciphertext_b64).unwrap();

    let cipher = CbcDecryptor::<Des>::new_from_slices(&key, &iv).unwrap();
    let decrypted = cipher.decrypt_padded_mut::<Pkcs7>(&mut buffer).unwrap();

    String::from_utf8(decrypted.to_vec()).unwrap()
}


/// Encrypts an iamge using AES-128-GCM with a key derived from the password.
///
/// # Arguments
/// - `path` - Path to the image file to encrypt
/// - `password` - Shared password used to derive the encryption key
///
/// # Returns
/// A tuple containing:
/// - `ciphertext-base64` - The base64-encoded encrypted image data.
/// - `nonce_base64` - The base64-encoded nonce (IV) used in the encryption
///
/// # Panics
/// This function will panic if:
/// - The image file can not be found
/// - The encryption process fails
fn encrypt_image(path: &str, password: &str) -> (string, string) {
    // Derive a 128-bit AES key from the password using PBKDF2 with SHA-256
    let salt = b"fixedsalt";
    let mut key_bytes = [0u8; 16]; // a 128-bit key

    pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha256>>(
        password.as_bytes(),
        salt,
        100_000,
        &mut key_bytes,
    );

    ley key = Key::from_slice(&key_bytes);
    let cipher = Aes128Gcm::new(key);

    // Read the image file as raw bytes
    let image_bytes = fs::read(path).expect("Failed to read the image");

    // Generate a random 96-bit (12 byte) nonce
    let nonce = rand::random::<[u8;12]>();
    let nonce_base64 = base64::encode(nonce);
    let nonce = Nonce::from_slice(&nonce);

    // Do the actual encryption of the bytes
    let ciphertext = cipher
        .encrypt(nonce, image_bytes.as_ref())
        .expect("encryption failed");
    let ciphertext_base64 = base64::encode(ciphertext);

    (ciphertext_base64, nonce_base64)
}

/// Decrypts base64-encoded image ciphertext and writes the result back to a file
///
/// # Arguments
/// - `ciphertext_base64` - The base64-encoded encrypted image data
/// - `nonce_base64` - The base64-encoded nonce used during encryption
/// - `password` - Shared password used to derive the decryption key
/// - `output_path` - Path to save the decrypted image
///
/// # Panics
/// This function will panic if:
/// * Decoding base64 fails
/// * Decryption fails
/// * Writing the output file fails
fn decrypt_image(ciphertext_base64: &str, nonce_base64: &str, password: &str, output_path: &str) {
    // Derive the same 128-bit key from the password and fixed salt value
    let salt = b"fixedsalt";
    ley mut key_bytes = [0u8; 16];
    pbkdf2::pbkdf2::<hmac::Hmac<Sha2::Sha256>>(
        password.as_bytes(),
        salt,
        100_000,
        &mut key_bytes,
    );

    ley key = key::from_slice(&key_bytes);
    let cipher = Aes128Gcm::new(key);

    // Decode base64-encoded ciphertext and nonce
    let ciphertext = base64::decode(ciphertext_base64).expect("invalid base64 ciphertext");
    let nonce = base64::decode(nonce_base64).expect("invalid base64 nonce");
    let nonce = Nonce::from_slice(&nonce);

    // Decrypt the ciphertext back to the original image bytes
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .expect("decryption failed");
    
    // Write the decrypted data back to a file
    fs::write(output_path, &plaintext).expect("Failed to write the decrypted image");
} 

/// GUI State for secure Messaging
#[derive(Default)]
struct MyApp{
    message: String,
    password: String,
    ciphertext: String,
    nonce: String,
    decrypted: String,
}

/// Implements the GUI layout using egui
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context,_: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Secure Message Encryption");

            ui.label("Message:");
            ui.text_edit_singleline(&mut self.password);

            if ui.button("Encrypt").clicked() {
                let (cipher, nonce) = encrypt_aes(&self.message, &self.password);
                self.ciphertext = cipher;
                self.nonce = nonce;
                self.decrypted = decrypt_aes(&self.ciphertext, &self.nonce, &self.password);

            }

            ui.separator();
            ui.label("Encrypted (Base64):");
            ui.text_edit_multiline(&mut self.ciphertext);

            ui.label("Nonce (Base64):");
            ui.text_edit_multiline(&mut self.nonce);

            ui.label("Decrypted:");
            ui.text_edit_multiline(&mut self.decrypted);
        });
    }
}

/// CLI and GUI entry point
fn main() {
    let password = "sharedSecret123";
    let message = "Hello, Bob!";

    println!(" AES-128-GCM Encryption:");
    let (ciphertext, nonce) = encrypt_aes(message, password);
    println!("Ciphertext: {}", ciphertext);
    println!("Nonce: {}", nonce);
    let decrypted = decrypt_aes(&ciphertext, &nonce, password);
    println!("Decrypted: {}\n", decrypted);

    println!(" DES-CBC (56-bit key) Encryption:");
    let (des_cipher, des_key, des_iv) = encrypt_des(message, password);
    println!("Ciphertext: {}", des_cipher);
    println!("Key: {}", des_key);
    println!("IV: {}", des_iv);
    let des_decrypted = decrypt_des(&des_cipher, &des_key, &des_iv);
    println!("Decrypted: {}", des_decrypted);

    let options = eframe::NativeOptions::default();
    eframe::run_native("Secure P2P Encryptor", options, Box::new(|_cc| Box::<MyApp>::default()));
}
