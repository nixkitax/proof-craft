// src/main.rs
fn main() {
    if let Err(e) = examples::demo::run() {
        eprintln!("Errore durante l'esecuzione dell'esempio: {}", e);
    }
}

mod examples {
    pub mod demo;
}