//! Estado e transições do Segundo Cérebro.

use chrono::{DateTime, Utc};
use estudy_domain::{EntityId, KnowledgeNote, NoteKind};
use iced::widget::text_editor;
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
#[derive(Debug, Clone)]
pub enum SecondBrainMessage {
    /// Filtra por tipo de artefato (`None` = Tudo).
    SetKindFilter(Option<NoteKind>),
    /// Filtra por tag (`None` = Todas).
    ToggleTagFilter(Option<String>),
    /// Atualiza o termo de busca instantânea.
    SearchChanged(String),
    /// Seleciona uma nota e abre o modal amplo de leitura/edição.
    SelectNote(EntityId),
    /// Cria uma nova nota em branco e já a abre no modal multi-linha.
    CreateNote,
    /// Fecha o modal de leitura/edição de nota.
    CloseNoteModal,
    /// Alterna aba `Escrever | Visualizar` no modal.
    SetEditorTab(BrainEditorTab),
    /// Atualiza o título da nota selecionada.
    TitleChanged(String),
    /// Atualiza o corpo Markdown da nota selecionada por string direta.
    BodyChanged(String),
    /// Processa ação multi-linha (`Enter`, digitação, seleção, colar) no `text_editor` da nota.
    BodyEditorAction(text_editor::Action),
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
#[derive(Debug)]
pub struct SecondBrainState {
    /// Notas armazenadas.
    pub notes: Vec<KnowledgeNote>,
    /// Filtro por tipo (`None` = Tudo).
    pub kind_filter: Option<NoteKind>,
    /// Filtro por tag ativa.
    pub tag_filter: Option<String>,
    /// Texto de busca instantânea.
    pub search_query: String,
    /// Nota atualmente selecionada.
    pub selected_note_id: Option<EntityId>,
    /// Indica se o modal amplo de leitura/edição está aberto.
    pub note_modal_open: bool,
    /// Buffer multi-linha real (`text_editor::Content`) sincronizado com a nota aberta.
    pub body_editor: text_editor::Content,
    /// Aba ativa do editor.
    pub editor_tab: BrainEditorTab,
    /// Campo temporário de nova tag.
    pub new_tag_input: String,
    dirty_note_ids: BTreeSet<EntityId>,
    deleted_note_ids: BTreeSet<EntityId>,
}

impl Clone for SecondBrainState {
    fn clone(&self) -> Self {
        Self {
            notes: self.notes.clone(),
            kind_filter: self.kind_filter,
            tag_filter: self.tag_filter.clone(),
            search_query: self.search_query.clone(),
            selected_note_id: self.selected_note_id.clone(),
            note_modal_open: self.note_modal_open,
            body_editor: text_editor::Content::with_text(&self.body_editor.text()),
            editor_tab: self.editor_tab,
            new_tag_input: self.new_tag_input.clone(),
            dirty_note_ids: self.dirty_note_ids.clone(),
            deleted_note_ids: self.deleted_note_ids.clone(),
        }
    }
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

        let body_editor =
            text_editor::Content::with_text(notes.first().map_or("", |n| n.body_md.as_str()));

        Self {
            notes,
            kind_filter: None,
            tag_filter: None,
            search_query: String::new(),
            selected_note_id: Some(first_id),
            note_modal_open: false,
            body_editor,
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
                if let Some(note) = self.notes.iter().find(|n| n.id == id) {
                    self.body_editor = text_editor::Content::with_text(&note.body_md);
                }
                self.selected_note_id = Some(id);
                self.note_modal_open = true;
                self.new_tag_input.clear();
            }
            SecondBrainMessage::CreateNote => self.create_new_note(now_utc),
            SecondBrainMessage::CloseNoteModal => {
                self.note_modal_open = false;
            }
            SecondBrainMessage::BodyEditorAction(action) => {
                self.body_editor.perform(action);
                let updated = self.body_editor.text().trim_end_matches('\n').to_string();
                self.mutate_selected_note(now_utc, |note| note.body_md = updated);
            }
            SecondBrainMessage::SetEditorTab(tab) => self.editor_tab = tab,
            SecondBrainMessage::TitleChanged(title) => {
                self.mutate_selected_note(now_utc, |note| note.title = title);
            }
            SecondBrainMessage::BodyChanged(body) => {
                self.body_editor = text_editor::Content::with_text(&body);
                self.mutate_selected_note(now_utc, |note| note.body_md = body);
            }
            SecondBrainMessage::KindChanged(kind) => {
                self.mutate_selected_note(now_utc, |note| note.kind = kind);
            }
            SecondBrainMessage::TogglePin(id) => {
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == id) {
                    note.pinned = !note.pinned;
                    note.updated_at = now_utc;
                    self.dirty_note_ids.insert(id);
                }
            }
            SecondBrainMessage::NewTagInputChanged(val) => self.new_tag_input = val,
            SecondBrainMessage::AddTag => self.add_tag_to_selected(now_utc),
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

    fn create_new_note(&mut self, now_utc: DateTime<Utc>) {
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
        self.body_editor = text_editor::Content::new();
        self.note_modal_open = true;
        self.editor_tab = BrainEditorTab::Write;
    }

    fn mutate_selected_note(
        &mut self,
        now_utc: DateTime<Utc>,
        apply: impl FnOnce(&mut KnowledgeNote),
    ) {
        if let Some(id) = self.selected_note_id.clone()
            && let Some(note) = self.notes.iter_mut().find(|n| n.id == id)
        {
            apply(note);
            note.updated_at = now_utc;
            self.dirty_note_ids.insert(id);
        }
    }

    fn add_tag_to_selected(&mut self, now_utc: DateTime<Utc>) {
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
}
