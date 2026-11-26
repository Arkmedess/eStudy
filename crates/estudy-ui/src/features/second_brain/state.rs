//! Estado e transições do Segundo Cérebro.

use chrono::{DateTime, Utc};
use estudy_domain::{EntityId, KnowledgeNote, NoteKind};
use std::collections::BTreeSet;

/// Aba ativa do painel lateral de edição/leitura de nota.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrainEditorTab {
    /// Modo de escrita Markdown.
    Write,
    /// Modo de visualização limpa.
    Preview,
}

/// Mensagens da fatia Segundo Cérebro.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecondBrainMessage {
    /// Filtra por tipo de artefato (`None` = Tudo).
    SetKindFilter(Option<NoteKind>),
    /// Filtra por tag (`None` = Todas).
    ToggleTagFilter(Option<String>),
    /// Atualiza o termo de busca instantânea.
    SearchChanged(String),
    /// Seleciona uma nota para abrir no painel de leitura/edição à direita.
    SelectNote(EntityId),
    /// Cria uma nova nota em branco e já a abre para edição.
    CreateNote,
    /// Alterna aba `Escrever | Visualizar` no painel lateral.
    SetEditorTab(BrainEditorTab),
    /// Atualiza o título da nota selecionada.
    TitleChanged(String),
    /// Atualiza o corpo Markdown da nota selecionada.
    BodyChanged(String),
    /// Altera o tipo (`Nota / Ideia / Referência / Link`) da nota selecionada.
    KindChanged(NoteKind),
    /// Alterna fixação no topo da grade.
    TogglePin(EntityId),
    /// Atualiza texto do campo de nova tag.
    NewTagInputChanged(String),
    /// Adiciona tag à nota selecionada.
    AddTag,
    /// Remove a nota selecionada.
    DeleteNote(EntityId),
}

/// Estado completo do Segundo Cérebro.
#[derive(Debug, Clone)]
pub struct SecondBrainState {
    /// Notas armazenadas.
    pub notes: Vec<KnowledgeNote>,
    /// Filtro por tipo (`None` = Tudo).
    pub kind_filter: Option<NoteKind>,
    /// Filtro por tag ativa.
    pub tag_filter: Option<String>,
    /// Texto de busca instantânea.
    pub search_query: String,
    /// Nota atualmente aberta no painel de edição à direita.
    pub selected_note_id: Option<EntityId>,
    /// Aba ativa do editor lateral.
    pub editor_tab: BrainEditorTab,
    /// Campo temporário de nova tag.
    pub new_tag_input: String,
    dirty_note_ids: BTreeSet<EntityId>,
    deleted_note_ids: BTreeSet<EntityId>,
}

impl SecondBrainState {
    /// Cria uma base inicial de exemplo com notas, ideias, referências e links.
    #[must_use]
    pub fn sample(now_utc: DateTime<Utc>) -> Self {
        let first_id = EntityId::new_v7();
        let notes = vec![
            KnowledgeNote {
                id: first_id.clone(),
                title: "Princípios de UI Calma e Legível".to_string(),
                kind: NoteKind::Reference,
                body_md: "Hierarquia nasce de contraste tipográfico, alinhamento consistente e cor usada com intenção — nunca de bordas pesadas ou excesso de badges.".to_string(),
                source_url: None,
                tags: vec!["design".to_string(), "estudy".to_string()],
                pinned: true,
                created_at: now_utc,
                updated_at: now_utc,
            },
            KnowledgeNote {
                id: EntityId::new_v7(),
                title: "Resumo — Revisão Espaçada (1/7/14/30d)".to_string(),
                kind: NoteKind::Note,
                body_md: "Revisar ativamente antes da curva de esquecimento reduz o esforço total em até 60%. Combinar com blocos curtos de 25 minutos e registro de erros.".to_string(),
                source_url: None,
                tags: vec!["estudos".to_string(), "memoria".to_string()],
                pinned: false,
                created_at: now_utc,
                updated_at: now_utc,
            },
            KnowledgeNote {
                id: EntityId::new_v7(),
                title: "Ideia: Templates conectados de rotina".to_string(),
                kind: NoteKind::Idea,
                body_md: "Um botão rápido para instanciar um plano de estudo completo: cria tarefa, agenda primeira revisão e prepara o Pomodoro.".to_string(),
                source_url: None,
                tags: vec!["estudy".to_string(), "produto".to_string()],
                pinned: false,
                created_at: now_utc,
                updated_at: now_utc,
            },
            KnowledgeNote {
                id: EntityId::new_v7(),
                title: "Documentação oficial do Iced (Canvas & MVU)".to_string(),
                kind: NoteKind::Link,
                body_md: "Referência de arquitetura Elm/MVU, caches de Canvas e otimização de renderização com tiny-skia em Rust.".to_string(),
                source_url: Some("https://docs.rs/iced".to_string()),
                tags: vec!["rust".to_string(), "iced".to_string()],
                pinned: false,
                created_at: now_utc,
                updated_at: now_utc,
            },
        ];

        Self {
            notes,
            kind_filter: None,
            tag_filter: None,
            search_query: String::new(),
            selected_note_id: Some(first_id),
            editor_tab: BrainEditorTab::Preview,
            new_tag_input: String::new(),
            dirty_note_ids: BTreeSet::new(),
            deleted_note_ids: BTreeSet::new(),
        }
    }

    /// Consome e retorna os conjuntos de notas alteradas e removidas para persistência seletiva.
    pub fn take_dirty(&mut self) -> (Vec<EntityId>, Vec<EntityId>) {
        let dirty = self.dirty_note_ids.iter().cloned().collect();
        let deleted = self.deleted_note_ids.iter().cloned().collect();
        self.dirty_note_ids.clear();
        self.deleted_note_ids.clear();
        (dirty, deleted)
    }

    /// Lista ordenada de todas as tags únicas presentes nas notas.
    #[must_use]
    pub fn available_tags(&self) -> Vec<String> {
        let mut set = BTreeSet::new();
        for note in &self.notes {
            for tag in &note.tags {
                set.insert(tag.clone());
            }
        }
        set.into_iter().collect()
    }

    /// Retorna as notas filtradas com notas fixadas primeiro.
    #[must_use]
    pub fn filtered_notes(&self) -> Vec<&KnowledgeNote> {
        let mut list: Vec<&KnowledgeNote> = self
            .notes
            .iter()
            .filter(|note| {
                note.matches_filter(
                    self.kind_filter,
                    self.tag_filter.as_deref(),
                    &self.search_query,
                )
            })
            .collect();
        list.sort_by_key(|note| !note.pinned);
        list
    }

    /// Retorna a nota selecionada atualmente para o painel de edição/leitura.
    #[must_use]
    pub fn selected_note(&self) -> Option<&KnowledgeNote> {
        let id = self.selected_note_id.as_ref()?;
        self.notes.iter().find(|n| &n.id == id)
    }

    /// Processa uma mensagem MVU atualizando o estado do Segundo Cérebro.
    pub fn update(&mut self, message: SecondBrainMessage, now_utc: DateTime<Utc>) {
        match message {
            SecondBrainMessage::SetKindFilter(kind) => self.kind_filter = kind,
            SecondBrainMessage::ToggleTagFilter(tag) => self.tag_filter = tag,
            SecondBrainMessage::SearchChanged(query) => self.search_query = query,
            SecondBrainMessage::SelectNote(id) => {
                self.selected_note_id = Some(id);
                self.new_tag_input.clear();
            }
            SecondBrainMessage::CreateNote => {
                let id = EntityId::new_v7();
                let kind = self.kind_filter.unwrap_or(NoteKind::Note);
                self.notes.insert(
                    0,
                    KnowledgeNote {
                        id: id.clone(),
                        title: "Nova nota".to_string(),
                        kind,
                        body_md: String::new(),
                        source_url: None,
                        tags: Vec::new(),
                        pinned: false,
                        created_at: now_utc,
                        updated_at: now_utc,
                    },
                );
                self.dirty_note_ids.insert(id.clone());
                self.selected_note_id = Some(id);
                self.editor_tab = BrainEditorTab::Write;
            }
            SecondBrainMessage::SetEditorTab(tab) => self.editor_tab = tab,
            SecondBrainMessage::TitleChanged(title) => {
                if let Some(id) = self.selected_note_id.clone()
                    && let Some(note) = self.notes.iter_mut().find(|n| n.id == id)
                {
                    note.title = title;
                    note.updated_at = now_utc;
                    self.dirty_note_ids.insert(id);
                }
            }
            SecondBrainMessage::BodyChanged(body) => {
                if let Some(id) = self.selected_note_id.clone()
                    && let Some(note) = self.notes.iter_mut().find(|n| n.id == id)
                {
                    note.body_md = body;
                    note.updated_at = now_utc;
                    self.dirty_note_ids.insert(id);
                }
            }
            SecondBrainMessage::KindChanged(kind) => {
                if let Some(id) = self.selected_note_id.clone()
                    && let Some(note) = self.notes.iter_mut().find(|n| n.id == id)
                {
                    note.kind = kind;
                    note.updated_at = now_utc;
                    self.dirty_note_ids.insert(id);
                }
            }
            SecondBrainMessage::TogglePin(id) => {
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                    note.pinned = !note.pinned;
                    note.updated_at = now_utc;
                    self.dirty_note_ids.insert(id);
                }
            }
            SecondBrainMessage::NewTagInputChanged(val) => self.new_tag_input = val,
            SecondBrainMessage::AddTag => {
                let clean = self
                    .new_tag_input
                    .trim()
                    .trim_start_matches('#')
                    .to_string();
                if clean.is_empty() {
                    return;
                }
                if let Some(id) = self.selected_note_id.clone()
                    && let Some(note) = self.notes.iter_mut().find(|n| n.id == id)
                    && !note.tags.iter().any(|t| t.eq_ignore_ascii_case(&clean))
                {
                    note.tags.push(clean);
                    note.updated_at = now_utc;
                    self.dirty_note_ids.insert(id);
                }
                self.new_tag_input.clear();
            }
            SecondBrainMessage::DeleteNote(id) => {
                self.notes.retain(|n| n.id != id);
                self.dirty_note_ids.remove(&id);
                self.deleted_note_ids.insert(id.clone());
                if self.selected_note_id.as_ref() == Some(&id) {
                    self.selected_note_id = self.notes.first().map(|n| n.id.clone());
                }
            }
        }
    }
}
