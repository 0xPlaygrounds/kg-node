use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

#[test]
fn main() {
    let embedding_model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true),
    )
    .expect("Failed to initialize embedding model");

    let embeddings = embedding_model
        .embed(vec!["Fries", "Chips", "Frites", "Fries Chips Frites"], None)
        .expect("Failed to embed texts");

    // for (text, embedding) in ["Fries", "Chips", "Frites", "Fries Chips Frites"]
    //     .iter()
    //     .zip(embeddings)
    // {
    //     println!("Text: {}, Embedding: {:?}", text, embedding);
    // }

    // // Calculate and print the pairwise cosine distances in a 4x4 matrix
    // let embeddings: Vec<_> = embedding_model
    //     .embed_texts(&["Fries", "Chips", "Frites", "Fries Chips Frites"])
    //     .expect("Failed to embed texts");

    println!("\nCosine distance matrix:");
    for i in 0..embeddings.len() {
        for j in 0..embeddings.len() {
            let dist = cosine_distance(&embeddings[i], &embeddings[j]);
            print!("{:.4} ", dist);
        }
        println!();
    }
}

fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    1.0 - (dot_product / (norm_a * norm_b))
}
