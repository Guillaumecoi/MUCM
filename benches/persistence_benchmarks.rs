use criterion::{criterion_group, criterion_main, Criterion};
use markdown_use_case_manager::config::{Config, StorageBackend};
use markdown_use_case_manager::core::{RepositoryFactory, UseCase, UseCaseRepository};
use tempfile::TempDir;

/// Create a test use case with the given ID
fn create_test_use_case(id: &str, title: &str, category: &str) -> UseCase {
    UseCase::new(
        id.to_string(),
        title.to_string(),
        category.to_string(),
        "TES".to_string(),
        "A test use case for benchmarking".to_string(),
        "medium".to_string(),
    )
    .expect("Failed to create test use case")
}

/// Create multiple test use cases
fn create_test_use_cases(count: usize) -> Vec<UseCase> {
    (0..count)
        .map(|i| {
            create_test_use_case(
                &format!("UC-BENCH-{:03}", i + 1),
                &format!("Benchmark Use Case {}", i + 1),
                "benchmark",
            )
        })
        .collect()
}

/// Benchmark setup for a specific backend
fn setup_backend(
    backend: StorageBackend,
    temp_dir: &TempDir,
) -> (Config, Box<dyn UseCaseRepository>) {
    // Create a minimal config
    let mut config = Config::default();
    config.storage.backend = backend;

    // Set up directories in temp dir
    config.directories.use_case_dir = temp_dir
        .path()
        .join("use-cases")
        .to_string_lossy()
        .to_string();
    config.directories.data_dir = temp_dir.path().join("data").to_string_lossy().to_string();

    let repository = if backend == StorageBackend::Sqlite {
        let db_path = temp_dir.path().join("benchmark.db");
        RepositoryFactory::create_with_db_path(&config, &db_path)
            .expect("Failed to create repository")
    } else {
        RepositoryFactory::create(&config).expect("Failed to create repository")
    };

    (config, repository)
}

/// Benchmark save operations
fn bench_save(c: &mut Criterion, backend: StorageBackend, use_case_count: usize) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let (_config, repository) = setup_backend(backend, &temp_dir);
    let use_cases = create_test_use_cases(use_case_count);

    let backend_name = match backend {
        StorageBackend::Toml => "toml",
        StorageBackend::Sqlite => "sqlite",
    };

    c.bench_function(
        &format!("save_{}_{}_use_cases", backend_name, use_case_count),
        |b| {
            b.iter(|| {
                for use_case in &use_cases {
                    std::hint::black_box(repository.save(use_case)).expect("Save failed");
                }
            })
        },
    );
}

/// Benchmark load_all operations
fn bench_load_all(c: &mut Criterion, backend: StorageBackend, use_case_count: usize) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let (_config, repository) = setup_backend(backend, &temp_dir);
    let use_cases = create_test_use_cases(use_case_count);

    // Pre-populate the repository
    for use_case in &use_cases {
        repository.save(use_case).expect("Pre-save failed");
    }

    let backend_name = match backend {
        StorageBackend::Toml => "toml",
        StorageBackend::Sqlite => "sqlite",
    };

    c.bench_function(
        &format!("load_all_{}_{}_use_cases", backend_name, use_case_count),
        |b| {
            b.iter(|| {
                let _result = std::hint::black_box(repository.load_all()).expect("Load all failed");
            })
        },
    );
}

/// Benchmark load_by_id operations
fn bench_load_by_id(c: &mut Criterion, backend: StorageBackend, use_case_count: usize) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let (_config, repository) = setup_backend(backend, &temp_dir);
    let use_cases = create_test_use_cases(use_case_count);

    // Pre-populate the repository
    for use_case in &use_cases {
        repository.save(use_case).expect("Pre-save failed");
    }

    let backend_name = match backend {
        StorageBackend::Toml => "toml",
        StorageBackend::Sqlite => "sqlite",
    };

    c.bench_function(
        &format!("load_by_id_{}_{}_use_cases", backend_name, use_case_count),
        |b| {
            b.iter(|| {
                for use_case in &use_cases {
                    let _result = std::hint::black_box(repository.load_by_id(&use_case.id))
                        .expect("Load by ID failed");
                }
            })
        },
    );
}

/// Benchmark filter by category operations (load_all + filter)
fn bench_find_by_category(c: &mut Criterion, backend: StorageBackend, use_case_count: usize) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let (_config, repository) = setup_backend(backend, &temp_dir);
    let use_cases = create_test_use_cases(use_case_count);

    // Pre-populate the repository
    for use_case in &use_cases {
        repository.save(use_case).expect("Pre-save failed");
    }

    let backend_name = match backend {
        StorageBackend::Toml => "toml",
        StorageBackend::Sqlite => "sqlite",
    };

    c.bench_function(
        &format!(
            "filter_by_category_{}_{}_use_cases",
            backend_name, use_case_count
        ),
        |b| {
            b.iter(|| {
                let all_use_cases =
                    std::hint::black_box(repository.load_all()).expect("Load all failed");
                let _filtered: Vec<_> = all_use_cases
                    .iter()
                    .filter(|uc| uc.category == "benchmark")
                    .collect();
            })
        },
    );
}

/// Run all benchmarks for a specific backend and use case count
fn bench_backend(c: &mut Criterion, backend: StorageBackend, use_case_count: usize) {
    bench_save(c, backend, use_case_count);
    bench_load_all(c, backend, use_case_count);
    bench_load_by_id(c, backend, use_case_count);
    bench_find_by_category(c, backend, use_case_count);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_toml_backend_small_dataset() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let (_config, repository) = setup_backend(StorageBackend::Toml, &temp_dir);
        let use_cases = create_test_use_cases(10);

        // Test save
        for use_case in &use_cases {
            repository.save(use_case).expect("Save failed");
        }

        // Test load all
        let loaded = repository.load_all().expect("Load all failed");
        assert_eq!(loaded.len(), 10);

        // Test load by id
        for use_case in &use_cases {
            let loaded = repository
                .load_by_id(&use_case.id)
                .expect("Load by ID failed");
            assert!(loaded.is_some());
        }

        // Test filter by category (using load_all + filter)
        let all_use_cases = repository.load_all().expect("Load all failed");
        let found: Vec<_> = all_use_cases
            .iter()
            .filter(|uc| uc.category == "benchmark")
            .collect();
        assert_eq!(found.len(), 10);
    }

    #[test]
    #[serial]
    fn test_sqlite_backend_small_dataset() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let (_config, repository) = setup_backend(StorageBackend::Sqlite, &temp_dir);
        let use_cases = create_test_use_cases(10);

        // Test save
        for use_case in &use_cases {
            repository.save(use_case).expect("Save failed");
        }

        // Test load all
        let loaded = repository.load_all().expect("Load all failed");
        assert_eq!(loaded.len(), 10);

        // Test load by id
        for use_case in &use_cases {
            let loaded = repository
                .load_by_id(&use_case.id)
                .expect("Load by ID failed");
            assert!(loaded.is_some());
        }

        // Test filter by category (using load_all + filter)
        let all_use_cases = repository.load_all().expect("Load all failed");
        let found: Vec<_> = all_use_cases
            .iter()
            .filter(|uc| uc.category == "benchmark")
            .collect();
        assert_eq!(found.len(), 10);
    }
}

// Criterion benchmark functions
fn bench_small_datasets(c: &mut Criterion) {
    bench_backend(c, StorageBackend::Toml, 10);
    bench_backend(c, StorageBackend::Sqlite, 10);
}

fn bench_medium_datasets(c: &mut Criterion) {
    bench_backend(c, StorageBackend::Toml, 100);
    bench_backend(c, StorageBackend::Sqlite, 100);
}

fn bench_large_datasets(c: &mut Criterion) {
    bench_backend(c, StorageBackend::Toml, 500);
    bench_backend(c, StorageBackend::Sqlite, 500);
}

criterion_group!(
    name = persistence_benchmarks;
    config = Criterion::default().sample_size(10);
    targets = bench_small_datasets, bench_medium_datasets, bench_large_datasets
);
criterion_main!(persistence_benchmarks);
