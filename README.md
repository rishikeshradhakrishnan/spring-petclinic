# PetClinic - Rust Edition

A Rust port of the classic [Spring PetClinic](https://github.com/spring-projects/spring-petclinic) sample application.

## Tech Stack

- **Web Framework**: [Actix-web](https://actix.rs/) 4
- **Template Engine**: [Tera](https://keats.github.io/tera/) (Jinja2-style)
- **Database**: SQLite via [SQLx](https://github.com/launchbadge/sqlx)
- **Serialization**: [Serde](https://serde.rs/)
- **Date Handling**: [Chrono](https://github.com/chronotope/chrono)

## Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))

## Running the Application

```bash
cargo run
```

The application starts on **http://localhost:8080**.

The SQLite database (`petclinic.db`) is created automatically on first run with seed data.

## Project Structure

```
src/
  main.rs              # Application entry point and route configuration
  db.rs                # Database initialization and connection pool
  models.rs            # Domain models and form types
  controllers/         # HTTP request handlers
    owner_controller   # Owner CRUD operations
    pet_controller     # Pet CRUD operations
    visit_controller   # Visit creation
    vet_controller     # Vet listing (HTML + JSON)
    welcome_controller # Home page and error trigger
  repositories/        # Database access layer
    owner_repository   # Owner queries
    pet_repository     # Pet and PetType queries
    vet_repository     # Vet and Specialty queries
    visit_repository   # Visit queries
templates/             # Tera HTML templates
static/                # CSS, images, fonts
db/                    # SQL schema and seed data
```

## Features

- View and search owners by last name (with pagination)
- Create and edit owners
- Add and edit pets for owners
- Record veterinary visits
- View veterinarian list with specialties (HTML and JSON)
- Form validation with error display
- Bootstrap 5 responsive UI

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Welcome page |
| GET | `/owners/find` | Find owners form |
| GET | `/owners?lastName=...&page=...` | Search owners |
| GET | `/owners/new` | New owner form |
| POST | `/owners/new` | Create owner |
| GET | `/owners/{id}` | Owner details |
| GET | `/owners/{id}/edit` | Edit owner form |
| POST | `/owners/{id}/edit` | Update owner |
| GET | `/owners/{id}/pets/new` | New pet form |
| POST | `/owners/{id}/pets/new` | Create pet |
| GET | `/owners/{id}/pets/{petId}/edit` | Edit pet form |
| POST | `/owners/{id}/pets/{petId}/edit` | Update pet |
| GET | `/owners/{id}/pets/{petId}/visits/new` | New visit form |
| POST | `/owners/{id}/pets/{petId}/visits/new` | Create visit |
| GET | `/vets.html` | Vet list (HTML) |
| GET | `/vets` | Vet list (JSON) |
| GET | `/oups` | Error page demo |

## License

See [LICENSE.txt](LICENSE.txt).
