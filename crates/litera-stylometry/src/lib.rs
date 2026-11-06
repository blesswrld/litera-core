//! `litera-stylometry`: Quantitative Literary Stylometry, Information Entropy,
//! Authorial Fingerprint Matching (Burrows's Delta), Syntactic & Rhetorical Cadence,
//! Quintilian & Behaghel Taxis Figures, KWIC Concordance, and Stochastic Prose Synthesis.

pub mod concordance;
pub mod corpus;
pub mod discourse_coherence_graph;
pub mod entropy;
pub mod markov;
pub mod ngram_language_model;
pub mod readability;
pub mod rhetoric_taxonomy;
pub mod syntax_tree;

pub use concordance::{
    build_kwic_concordance, detect_rhyme_echoes, unicode_levenshtein_distance,
    KwicConcordanceLine, RhymeEchoPair, RhymeQuality,
};
pub use corpus::{
    compute_author_proximities, AuthorCentroid, AuthorProximityMatch, ClassicalAuthorId,
    StylometricVector,
};
pub use discourse_coherence_graph::{
    analyze_discourse_coherence, extract_morphological_stem, extract_sentence_content_stems,
    DiscourseCoherenceReport, SentenceCohesionEdge,
};
pub use entropy::{
    compute_lexical_entropy, EntropyAndRichnessReport, FrequencyRankEntry, ZipfDistributionFit,
};
pub use markov::{
    generate_stochastic_stanza, MarkovChainSynthesizer, StanzaGenerationTheme, SynthesizedStanza,
};
pub use ngram_language_model::{
    classify_orthographic_register, evaluate_ngram_language_model, NgramLanguageReport,
    OrthographicRegister,
};
pub use readability::{
    compute_readability_metrics, AudienceRegister, ReadabilityReport, SyntacticComplexityClass,
};
pub use rhetoric_taxonomy::{
    scan_classical_taxis_figures, ClassicalTaxisKind, TaxisFigureInstance, ANTITHESIS_POLAR_STEMS,
};
pub use syntax_tree::{
    analyze_syntactic_cadence, DetectedRhetoricalFigure, PunctuationCadenceProfile,
    RhetoricalFigureKind,
};
