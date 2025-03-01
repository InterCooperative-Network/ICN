use backend::{
    test_utils::TestServices,
    models::{Proposal, Vote},
};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chrono::Utc;
use futures::executor::block_on;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;

fn create_test_proposal(id: i32) -> Proposal {
    Proposal {
        id,
        title: format!("Benchmark Proposal {}", id),
        description: "Description for benchmark proposal".to_string(),
        created_by: "did:icn:test".to_string(),
        ends_at: Utc::now().naive_utc() + chrono::Duration::hours(1),
        created_at: Utc::now().naive_utc(),
    }
}

fn create_test_vote(proposal_id: i32, voter_id: i32) -> Vote {
    Vote {
        proposal_id,
        voter: format!("did:icn:voter{}", voter_id),
        approve: true,
    }
}

fn benchmark_proposal_creation(c: &mut Criterion) {
    let services = block_on(TestServices::new());
    
    c.bench_function("create single proposal", |b| {
        b.iter(|| {
            let proposal = create_test_proposal(black_box(1));
            block_on(services.database.create_proposal(&proposal))
        });
    });

    // Benchmark batch proposal creation
    c.bench_function("create 100 proposals", |b| {
        b.iter(|| {
            for i in 0..100 {
                let proposal = create_test_proposal(black_box(i));
                block_on(services.database.create_proposal(&proposal)).unwrap();
            }
        });
    });

    block_on(services.cleanup());
}

fn benchmark_vote_recording(c: &mut Criterion) {
    let services = block_on(TestServices::new());
    
    // Create test proposal
    let proposal = create_test_proposal(1);
    block_on(services.database.create_proposal(&proposal)).unwrap();

    c.bench_function("record single vote", |b| {
        b.iter(|| {
            let vote = create_test_vote(1, black_box(1));
            block_on(services.database.record_vote(&vote))
        });
    });

    // Benchmark batch vote recording
    c.bench_function("record 1000 votes", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let vote = create_test_vote(1, black_box(i));
                block_on(services.database.record_vote(&vote)).unwrap();
            }
        });
    });

    block_on(services.cleanup());
}

fn benchmark_proposal_queries(c: &mut Criterion) {
    let services = block_on(TestServices::new());
    
    // Create test data
    for i in 0..100 {
        let proposal = create_test_proposal(i);
        block_on(services.database.create_proposal(&proposal)).unwrap();
        
        // Add some votes
        for j in 0..10 {
            let vote = create_test_vote(i, j);
            block_on(services.database.record_vote(&vote)).unwrap();
        }
    }

    let mut group = c.benchmark_group("proposal_queries");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    group.bench_function("get all proposals", |b| {
        b.iter(|| {
            block_on(services.database.get_all_proposals())
        });
    });

    group.bench_function("get proposal votes", |b| {
        b.iter(|| {
            block_on(services.database.get_proposal_votes(black_box(1)))
        });
    });

    group.bench_function("get active proposals", |b| {
        b.iter(|| {
            block_on(services.database.get_active_proposals())
        });
    });

    group.finish();
    block_on(services.cleanup());
}

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let services = block_on(TestServices::new());
    let runtime = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("concurrent proposal creation", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let futures: Vec<_> = (0..100).map(|i| {
                    let proposal = create_test_proposal(i);
                    services.database.create_proposal(&proposal)
                }).collect();
                futures::future::join_all(futures).await
            })
        });
    });

    c.bench_function("concurrent vote recording", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let futures: Vec<_> = (0..1000).map(|i| {
                    let vote = create_test_vote(1, i);
                    services.database.record_vote(&vote)
                }).collect();
                futures::future::join_all(futures).await
            })
        });
    });

    block_on(services.cleanup());
}

pub fn governance_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Setup test database
    let db_pool = rt.block_on(async {
        // Setup test database connection
        sqlx::PgPool::connect("postgres://test:test@localhost/test_db").await.unwrap()
    });
    
    let service = Arc::new(Mutex::new(GovernanceService::new(Arc::new(db_pool))));
    
    // Benchmark proposal creation
    c.bench_function("create_proposal", |b| {
        b.iter(|| {
            rt.block_on(async {
                let proposal = Proposal {
                    title: "Test Proposal".to_string(),
                    description: "Test Description".to_string(),
                    created_by: "test_user".to_string(),
                    ends_at: "2099-12-31T23:59:59Z".to_string(),
                };
                
                let mut s = service.lock().await;
                s.create_proposal(black_box(proposal)).await.unwrap()
            })
        })
    });
    
    // Benchmark voting
    c.bench_function("vote_on_proposal", |b| {
        b.iter(|| {
            rt.block_on(async {
                let vote = Vote {
                    proposal_id: "test_id".to_string(),
                    voter: "test_voter".to_string(),
                    approve: true,
                };
                
                let mut s = service.lock().await;
                s.record_vote(black_box(vote)).await.unwrap()
            })
        })
    });
    
    // Benchmark proposal counting
    c.bench_function("count_votes", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut s = service.lock().await;
                s.count_votes(black_box("test_id")).await.unwrap()
            })
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(2));
    targets = benchmark_proposal_creation,
             benchmark_vote_recording,
             benchmark_proposal_queries,
             benchmark_concurrent_operations,
             governance_benchmark
);
criterion_main!(benches);