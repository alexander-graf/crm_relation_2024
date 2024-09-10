use serde::{Serialize, Deserialize};
use std::path::Path;
use std::fs;
use std::io::Read;
use tokio_postgres::{NoTls, Error as PgError};

#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct DbConfig {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
}

pub async fn create_database(config: &DbConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Attempting to create database: {}", config.database);
    
    let postgres_conn_string = format!(
        "host={} port={} user=postgres dbname=postgres",
        config.host, config.port
    );

    println!("Connecting to PostgreSQL server as postgres user...");
    let (client, connection) = tokio_postgres::connect(&postgres_conn_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let query = "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = $1)";
    println!("Executing query: {}", query);
    let db_exists: bool = client.query_one(query, &[&config.database]).await?.get(0);

    if !db_exists {
        println!("Database does not exist. Creating...");
        let create_db_query = format!("CREATE DATABASE {}", config.database);
        println!("Executing query: {}", create_db_query);
        client.execute(&create_db_query, &[]).await?;
        println!("Database created successfully");

        let grant_query = format!("GRANT ALL PRIVILEGES ON DATABASE {} TO {}", config.database, config.username);
        println!("Executing query: {}", grant_query);
        client.execute(&grant_query, &[]).await?;
        println!("Privileges granted to user {}", config.username);
    } else {
        println!("Database already exists");
    }

    Ok(())
}

pub async fn create_database_structure(config: &DbConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating database structure for: {}", config.database);
    let conn_string = format!(
        "host={} port={} user={} password={} dbname={}",
        config.host, config.port, config.username, config.password, config.database
    );

    println!("Connecting to database as user {}...", config.username);
    let (client, connection) = tokio_postgres::connect(&conn_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    println!("Connected successfully. Creating tables...");
    let create_tables_query = "
        CREATE TABLE IF NOT EXISTS customers (
            customer_id SERIAL PRIMARY KEY,
            company_name VARCHAR(100),
            contact_name VARCHAR(100),
            contact_position VARCHAR(50),
            address TEXT,
            phone VARCHAR(20),
            email VARCHAR(100),
            website VARCHAR(100),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS invoices (
            invoice_id SERIAL PRIMARY KEY,
            customer_id INTEGER REFERENCES customers(customer_id),
            invoice_number VARCHAR(50) UNIQUE,
            invoice_date DATE,
            due_date DATE,
            total_amount DECIMAL(10,2),
            status VARCHAR(20),
            payment_method VARCHAR(50),
            notes TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ";
    println!("Executing query:\n{}", create_tables_query);
    client.batch_execute(create_tables_query).await?;

    println!("Database structure created successfully");
    Ok(())
}

pub async fn test_connection(config: &DbConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing connection to PostgreSQL server...");
    let conn_string = format!(
        "host={} port={} user={} password={} dbname=postgres",
        config.host, config.port, config.username, config.password
    );

    match tokio_postgres::connect(&conn_string, NoTls).await {
        Ok(_) => {
            println!("Connection successful!");
            Ok(())
        },
        Err(e) => {
            println!("Connection failed: {}", e);
            Err(Box::new(e))
        }
    }
}
