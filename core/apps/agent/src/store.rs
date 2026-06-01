use std::fs;

use gem_tracing::tracing::info;
use rig::vector_store::request::VectorSearchRequest;
use rig::vector_store::{InsertDocuments, VectorStoreIndex};
use rig::{Embed, embeddings::EmbeddingsBuilder};
use rig_fastembed::{Client as FastembedClient, EmbeddingModel as FastembedEmbeddingModel, FastembedModel};
use rig_sqlite::{Column, ColumnValue, SqliteVectorStore, SqliteVectorStoreTable};
use serde::{Deserialize, Serialize};
use tokio_rusqlite::Connection;

use crate::config::Settings;
use crate::preamble;

#[derive(Embed, Clone, Debug, Deserialize, Serialize)]
pub struct Memo {
    pub id: String,
    pub source: String,
    #[embed]
    pub content: String,
}

impl SqliteVectorStoreTable for Memo {
    fn name() -> &'static str {
        "memos"
    }
    fn schema() -> Vec<Column> {
        vec![Column::new("id", "TEXT PRIMARY KEY"), Column::new("source", "TEXT"), Column::new("content", "TEXT")]
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn column_values(&self) -> Vec<(&'static str, Box<dyn ColumnValue>)> {
        vec![
            ("id", Box::new(self.id.clone())),
            ("source", Box::new(self.source.clone())),
            ("content", Box::new(self.content.clone())),
        ]
    }
}

pub struct MemoryStore {
    store: SqliteVectorStore<FastembedEmbeddingModel, Memo>,
    model: FastembedEmbeddingModel,
}

impl MemoryStore {
    pub async fn open_and_index(settings: &Settings) -> crate::Result<Self> {
        register_sqlite_vec_extension();
        let fastembed_model = pick_fastembed_model(&settings.embedding.model)?;

        let data_dir = settings.data_dir();
        fs::create_dir_all(&data_dir).map_err(|e| format!("create agent data dir: {e}"))?;
        let db_path = data_dir.join("index.sqlite");
        info!(
            agent = %settings.agent_name,
            path = %db_path.display(),
            model = %settings.embedding.model,
            "opening vector store"
        );
        let conn = Connection::open(&db_path).await.map_err(|e| format!("open sqlite: {e}"))?;

        let fastembed_client = FastembedClient::new();
        let model = fastembed_client.embedding_model(&fastembed_model).map_err(|e| format!("loading fastembed model: {e}"))?;

        let store: SqliteVectorStore<_, Memo> = SqliteVectorStore::new(conn, &model).await.map_err(|e| format!("init sqlite store: {e}"))?;

        let docs: Vec<Memo> = preamble::indexable_files(settings)?
            .into_iter()
            .map(|f| Memo {
                id: f.source.clone(),
                source: f.source,
                content: f.content,
            })
            .collect();
        if !docs.is_empty() {
            info!(
                agent = %settings.agent_name,
                count = docs.len(),
                "indexing markdown files"
            );
            let embeddings = EmbeddingsBuilder::new(model.clone())
                .documents(docs)?
                .build()
                .await
                .map_err(|e| format!("building embeddings: {e}"))?;
            store.insert_documents(embeddings).await.map_err(|e| format!("inserting documents: {e}"))?;
        }
        Ok(Self { store, model })
    }

    pub async fn search(&self, query: &str, top_n: u32) -> crate::Result<Vec<Memo>> {
        let req = VectorSearchRequest::builder().samples(top_n as u64).query(query).build();
        let index = self.store.clone().index(self.model.clone());
        let hits: Vec<(f64, String, Memo)> = index.top_n::<Memo>(req).await?;
        Ok(hits.into_iter().map(|(_, _, m)| m).collect())
    }

    pub async fn save(&self, id: String, source: String, content: String) -> crate::Result<()> {
        let memo = Memo { id, source, content };
        let embeddings = EmbeddingsBuilder::new(self.model.clone()).documents(vec![memo])?.build().await?;
        self.store.insert_documents(embeddings).await?;
        Ok(())
    }
}

fn pick_fastembed_model(name: &str) -> crate::Result<FastembedModel> {
    match name {
        "nomic-embed-text-v1.5-q" => Ok(FastembedModel::NomicEmbedTextV15Q),
        other => Err(format!("unsupported embedding.model: `{other}`").into()),
    }
}

fn register_sqlite_vec_extension() {
    use rusqlite::ffi::{sqlite3, sqlite3_api_routines, sqlite3_auto_extension};
    use sqlite_vec::sqlite3_vec_init;
    type ExtFn = unsafe extern "C" fn(*mut sqlite3, *mut *mut std::os::raw::c_char, *const sqlite3_api_routines) -> i32;
    unsafe {
        #[allow(clippy::missing_transmute_annotations)]
        sqlite3_auto_extension(Some(std::mem::transmute::<*const (), ExtFn>(sqlite3_vec_init as *const ())));
    }
}
