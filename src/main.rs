// src/main.rs
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Book {
    id: u64,
    title: String,
    author: String,
    // Add other fields as needed
}

#[derive(Debug, Serialize, Deserialize)]
struct BookInput {
    title: String,
    author: String,
    // Add other fields as needed
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: T,
}

// Shared state with Mutex for mutable access
#[derive(Clone)]
struct AppState {
    books: Arc<Mutex<Vec<Book>>>,
}

async fn get_books(state: web::Data<AppState>) -> impl Responder {
    let books = state.books.lock().unwrap(); // Lock the mutex
    let response = ApiResponse {
        success: true,
        data: books.clone(),
    };
    HttpResponse::Ok().json(response)
}

async fn add_book(book: web::Json<BookInput>, state: web::Data<AppState>) -> impl Responder {
    let new_book = Book {
        id: {
            let books = state.books.lock().unwrap(); // Lock the mutex
            books.len() as u64 + 1
        },
        title: book.title.clone(),
        author: book.author.clone(),
        // Initialize other fields
    };

    {
        let mut books = state.books.lock().unwrap(); // Lock the mutex
        books.push(new_book.clone());
    }

    let response = ApiResponse {
        success: true,
        data: new_book,
    };

    HttpResponse::Created().json(response)
}

async fn delete_book(path: web::Path<u64>, state: web::Data<AppState>) -> impl Responder {
    let book_id = path.into_inner();

    let mut books = state.books.lock().unwrap(); // Lock the mutex
    let index = books.iter().position(|book| book.id == book_id);

    if let Some(index) = index {
        let deleted_book = books.remove(index);

        let response = ApiResponse {
            success: true,
            data: deleted_book,
        };

        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::NotFound().json(ApiResponse {
            success: false,
            data: "Book not found",
        })
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize the in-memory state with a sample database of 3 books
    let app_state = web::Data::new(AppState {
        books: Arc::new(Mutex::new(vec![
            Book {
                id: 1,
                title: "The Rust Programming Language".to_string(),
                author: "Steve Klabnik".to_string(),
            },
            Book {
                id: 2,
                title: "Programming Rust".to_string(),
                author: "Jim Blandy".to_string(),
            },
            Book {
                id: 3,
                title: "Rust in Action".to_string(),
                author: "Tim McNamara".to_string(),
            },
        ])),
    });

    // Start the Actix-Web server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/books", web::get().to(get_books))
            .route("/books", web::post().to(add_book))
            .route("/books/{id}", web::delete().to(delete_book))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
