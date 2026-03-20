use crate::router::AppRouter;
use crate::state::{AppState, Command, Event, DockerfileEditor, LogsModal, SystemSnapshot, Theme, Toast, ToastType};
use dioxus::prelude::*;
use dioxus_signals::Signal;
use futures::channel::mpsc::UnboundedReceiver;
use futures::future::FutureExt;
use futures_util::{SinkExt, StreamExt};
use std::collections::VecDeque;
use std::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};

const LIGHT_THEME: &str = r#"
:root {
    --font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
    --bg: #f7f8fa;
    --surface: #ffffff;
    --panel: #ffffff;
    --panel-hover: #f7f8fa;
    --muted: #6c757d;
    --text: #212529;
    --accent: #0d6efd;
    --accent-hover: #0b5ed7;
    --border: #dee2e6;
    --border-light: #e9ecef;
    --success: #198754;
    --danger: #dc3545;
    --warning: #ffc107;
}
"#;

const DARK_THEME: &str = r#"
:root {
    --font-family: 'Inter', 'Segoe UI', system-ui, sans-serif;
    --bg: #1a1d23;
    --surface: #23262d;
    --panel: #23262d;
    --panel-hover: #2c2f38;
    --muted: #8b92a0;
    --text: #e4e6eb;
    --accent: #4dabf7;
    --accent-hover: #339af0;
    --border: #3a3d47;
    --border-light: #2f323a;
    --success: #51cf66;
    --danger: #ff6b6b;
    --warning: #ffd43b;
}
"#;

const BASE_STYLE: &str = r#"

* { box-sizing: border-box; margin: 0; padding: 0; }
html, body, #main { width: 100%; height: 100%; background: var(--bg); color: var(--text); font-family: var(--font-family); font-size: 14px; }

.app-shell { display: flex; width: 100%; height: 100%; }
.content-area { flex: 1; display: flex; flex-direction: column; background: var(--bg); overflow: hidden; }
.section-body { flex: 1; padding: 0; overflow: auto; }

/* Sidebar */
.sidebar {
    width: 240px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--surface);
    border-right: 1px solid var(--border);
}

.sidebar-brand {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 16px;
    border-bottom: 1px solid var(--border);
    font-size: 16px;
    font-weight: 600;
}

.sidebar-brand > span {
    flex: 1;
}

.brand-icon {
    font-size: 20px;
    color: var(--accent);
}

.sidebar-nav {
    display: flex;
    flex-direction: column;
    padding: 8px;
}

.sidebar-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 6px;
    text-decoration: none;
    color: var(--text);
    font-size: 14px;
    transition: all 0.15s;
}

.sidebar-item:hover {
    background: var(--panel-hover);
}

.sidebar-item-active {
    background: var(--panel-hover);
    color: var(--accent);
    font-weight: 500;
}

.sidebar-icon {
    font-size: 18px;
    width: 20px;
    text-align: center;
}

/* Docker View */
.docker-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
}

.view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
}

.view-title {
    display: flex;
    align-items: center;
    gap: 12px;
}

.view-title h1 {
    font-size: 24px;
    font-weight: 600;
}

/* Monitoring View */
.monitoring-dashboard {
    height: 100%;
    overflow: auto;
    padding: 24px;
}
.monitoring-grid {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 24px;
    height: 100%;
}
.monitoring-col-main {
    display: flex;
    flex-direction: column;
    gap: 24px;
}
.monitoring-col-side {
    display: flex;
    flex-direction: column;
    gap: 24px;
}

/* Resource Graphs */
.resource-graphs-container {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 20px;
}

.resource-graph-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-height: 250px;
}

.graph-card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    flex-wrap: wrap;
    gap: 8px;
}

.graph-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
    margin: 0;
}

.graph-stats {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 4px;
}

.current-value {
    font-size: 18px;
    font-weight: 700;
    color: var(--accent);
}

.avg-value {
    font-size: 12px;
    color: var(--muted);
}

.graph-container {
    flex: 1;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 12px;
    overflow: hidden;
}

.resource-graph {
    width: 100%;
    height: 100%;
}

.graph-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding-top: 8px;
    border-top: 1px solid var(--border);
}

.graph-label {
    font-size: 11px;
    color: var(--muted);
}

.graph-range {
    font-size: 11px;
    color: var(--muted);
    font-weight: 500;
}

.process-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
}
.process-table th {
    text-align: left;
    padding: 8px 4px;
    color: var(--muted);
    font-weight: 500;
    border-bottom: 1px solid var(--border);
}
.process-table td {
    padding: 8px 4px;
    border-bottom: 1px solid var(--border-light);
}
.panel {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
}
.panel h2 {
    font-size: 18px;
    font-weight: 600;
    margin: 0;
}
.grid-two {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
}
.stat {
    display: flex;
    flex-direction: column;
    gap: 4px;
}
.stat .label {
    font-size: 12px;
    color: var(--muted);
    text-transform: uppercase;
    font-weight: 600;
}
.stat .value {
    font-size: 24px;
    font-weight: 500;
    font-feature-settings: \"tnum\";
}
.muted {
    color: var(--muted);
}

.count-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 24px;
    height: 24px;
    padding: 0 8px;
    border-radius: 12px;
    background: var(--panel-hover);
    color: var(--muted);
    font-size: 13px;
    font-weight: 500;
}

.view-actions {
    display: flex;
    align-items: center;
    gap: 12px;
}

.search-input {
    width: 300px;
    padding: 8px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 14px;
}

.search-input:focus {
    outline: none;
    border-color: var(--accent);
}

.status-badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 16px;
    font-size: 12px;
    font-weight: 500;
}

.status-badge.status-running {
    color: var(--success);
    background: rgba(25, 135, 84, 0.1);
}

.status-badge.status-stopped {
    color: var(--muted);
    background: rgba(108, 117, 125, 0.1);
}

.status-badge.status-warning {
    color: var(--warning);
    background: rgba(255, 193, 7, 0.1);
}

.status-badge.status-error {
    color: var(--danger);
    background: rgba(220, 53, 69, 0.1);
}

/* Docker Table */
.docker-table {
    width: 100%;
    border-collapse: collapse;
    background: var(--surface);
}

.docker-table thead {
    position: sticky;
    top: 0;
    background: var(--surface);
    z-index: 10;
}

.docker-table th {
    padding: 12px 16px;
    text-align: left;
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    border-bottom: 1px solid var(--border);
}

.docker-table td {
    padding: 12px 16px;
    border-bottom: 1px solid var(--border-light);
    font-size: 13px;
}

.docker-table tbody tr {
    transition: background 0.1s;
}

.docker-table tbody tr:hover {
    background: var(--panel-hover);
}

.col-checkbox {
    width: 48px;
    text-align: center;
}

.col-name {
    min-width: 200px;
}

.cell-main {
    font-weight: 500;
    margin-bottom: 4px;
}

.cell-sub {
    color: var(--muted);
    font-size: 12px;
}

.col-image {
    color: var(--muted);
}

.col-status {
    width: 140px;
}

.col-ports {
    color: var(--muted);
}

.col-actions {
    width: 180px;
}

.action-buttons {
    display: flex;
    gap: 4px;
    opacity: 0;
    transition: opacity 0.15s;
}

.docker-table tbody tr:hover .action-buttons {
    opacity: 1;
}

.action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.15s;
}

.action-btn:hover {
    background: var(--panel-hover);
    border-color: var(--accent);
}

.action-btn.action-primary {
    color: var(--success);
}

.action-btn.action-primary:hover {
    background: var(--success);
    color: white;
    border-color: var(--success);
}

.action-btn.action-danger {
    color: var(--danger);
}

.action-btn.action-danger:hover {
    background: var(--danger);
    color: white;
    border-color: var(--danger);
}

/* Empty State */
.empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 64px 24px;
    text-align: center;
}

.empty-icon {
    font-size: 64px;
    color: var(--muted);
    opacity: 0.3;
    margin-bottom: 16px;
}

.empty-state h3 {
    font-size: 18px;
    font-weight: 600;
    margin-bottom: 8px;
}

.empty-state p {
    color: var(--muted);
    font-size: 14px;
}

/* Alert */
.alert {
    margin: 16px 24px;
    padding: 12px 16px;
    border-radius: 6px;
    font-size: 13px;
}

.alert-error {
    background: rgba(220, 53, 69, 0.1);
    color: var(--danger);
    border: 1px solid rgba(220, 53, 69, 0.2);
}

/* Legacy Compatibility */
.topbar { display: flex; align-items: center; justify-content: space-between; padding: 14px 20px; border-bottom: 1px solid var(--border); background: var(--surface); }
.topbar-title { font-size: 18px; font-weight: 600; }
.panel { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 16px; }
.panel h2 { margin: 0 0 12px; font-size: 16px; font-weight: 600; }
.muted { color: var(--muted); }

/* Buttons - comprehensive styling */
.btn, button.btn, button[class*="btn-"] {
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px 16px;
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-size: 14px;
    font-family: inherit;
    transition: all 0.15s;
}

.btn:hover, button.btn:hover, button[class*="btn-"]:hover {
    background: var(--panel-hover);
}

.btn.primary, .btn.btn-primary, .btn-primary {
    background: var(--accent);
    color: #fff;
    border-color: var(--accent);
    font-weight: 500;
}

.btn.primary:hover, .btn.btn-primary:hover, .btn-primary:hover {
    background: var(--accent-hover);
    border-color: var(--accent-hover);
}

.btn.btn-secondary, .btn-secondary {
    background: var(--surface);
    color: var(--text);
    border-color: var(--border);
}

.btn.btn-secondary:hover, .btn-secondary:hover {
    background: var(--panel-hover);
    border-color: var(--border-light);
}

.btn.btn-outline, .btn-outline {
    background: transparent;
    border-color: var(--border);
    color: var(--text);
}

.btn-outline:hover {
    background: var(--panel-hover);
}

.btn:disabled, button[class*="btn-"]:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn.disabled, .btn[disabled] {
    opacity: 0.5;
    cursor: not-allowed;
    pointer-events: none;
}

.input { width: 100%; padding: 8px 12px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface); color: var(--text); font-size: 14px; }
.input:focus { outline: none; border-color: var(--accent); }
textarea.input {
    resize: vertical;
    font-family: var(--font-family);
    min-height: 60px;
}
select.input {
    appearance: auto;
}

/* Modal */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
}

.modal {
    background: var(--surface);
    border-radius: 8px;
    padding: 24px;
    max-width: 500px;
    width: 90%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
}

.wizard-modal {
    background: var(--surface);
    border-radius: 12px;
    padding: 24px;
    max-width: 900px;
    width: 95%;
    max-height: 90vh;
    overflow-y: auto;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.35);
}

.wizard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
}

.close-btn {
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    border-radius: 6px;
    padding: 4px 10px;
    cursor: pointer;
}

.wizard-progress {
    display: grid;
    gap: 8px;
    margin-bottom: 16px;
}

.progress-step {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--muted);
    background: var(--surface);
}

.step-title { font-weight: 500; }

.progress-step.active {
    border-color: var(--accent);
    color: var(--text);
    background: rgba(88, 101, 242, 0.12);
}

.progress-step.completed {
    border-color: rgba(34, 197, 94, 0.6);
    color: var(--text);
    background: rgba(34, 197, 94, 0.12);
}

.step-number {
    width: 24px;
    height: 24px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    border: 1px solid var(--border);
}

.wizard-content {
    display: grid;
    gap: 16px;
}

.wizard-actions {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin-top: 24px;
}

.wizard-steps {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 16px;
}

/* Virtual Environments list */
.panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
}

.panel-actions { display: flex; gap: 8px; }

.panel-content { display: grid; gap: 12px; }

.environment-list {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.environment-row {
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.5rem;
    transition: all 0.2s ease;
    cursor: pointer;
}

.environment-row:hover {
    background: var(--panel-hover);
    border-color: var(--accent);
}

.environment-row.active {
    border-color: var(--success);
    background: color-mix(in srgb, var(--success) 10%, var(--panel));
}

.env-main-info {
    display: flex;
    flex-direction: column;
    gap: 1rem;
}

.env-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
}

.env-name-section {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

.env-name {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
    color: var(--text);
}

.env-badges {
    display: flex;
    gap: 0.5rem;
    align-items: center;
}

.language-badge {
    padding: 0.25rem 0.75rem;
    border-radius: 4px;
    font-size: 0.875rem;
    font-weight: 500;
    background: var(--border);
    color: var(--text);
}

.language-badge.python {
    background: #3776ab;
    color: white;
}

.language-badge.node {
    background: #339933;
    color: white;
}

.language-badge.rust {
    background: #ce422b;
    color: white;
}

.language-badge.java {
    background: #ed8b00;
    color: white;
}

.status-badge.active {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    background: var(--success);
    color: #000;
}

.env-actions {
    display: flex;
    gap: 0.5rem;
    align-items: flex-start;
    flex-wrap: wrap;
}

.env-details {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
}

.detail-row {
    display: flex;
    gap: 2rem;
    flex-wrap: wrap;
}

.detail-item {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    min-width: 200px;
}

.detail-item .label {
    font-weight: 500;
    color: var(--muted);
    min-width: 80px;
}

.detail-item .value {
    color: var(--text);
}

.detail-item .value.path {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.875rem;
    background: var(--border);
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    max-width: 400px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.detail-item .value.template {
    font-style: italic;
    color: var(--accent);
}

.summary-info {
    color: var(--muted);
    font-size: 12px;
}

.wizard-step {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 16px;
}

.wizard-step.active {
    background: rgba(88, 101, 242, 0.12);
    border-color: var(--accent);
    color: var(--text);
}

.wizard-body {
    display: grid;
    gap: 16px;
}

.wizard-step h3 {
    margin: 0 0 12px;
    font-size: 16px;
}

.template-options {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
    gap: 12px;
}

.template-card {
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 12px;
    cursor: pointer;
    background: var(--surface);
    transition: border-color 0.15s ease, background 0.15s ease, transform 0.15s ease;
}

.template-card:hover {
    border-color: var(--accent);
    background: var(--panel-hover);
    transform: translateY(-1px);
}

.template-card.selected {
    border-color: var(--accent);
    background: rgba(88, 101, 242, 0.12);
}

.template-name { font-weight: 600; margin-bottom: 6px; }
.template-description { font-size: 12px; color: var(--muted); margin-bottom: 8px; }
.template-packages { font-size: 12px; color: var(--text); }

.package-input {
    display: flex;
    gap: 10px;
    align-items: center;
}

.package-list {
    margin-top: 12px;
    display: grid;
    gap: 8px;
}

.package-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
}

.remove-btn {
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text);
    border-radius: 6px;
    padding: 2px 8px;
    cursor: pointer;
}

.package-suggestions {
    margin-top: 10px;
}

.package-suggestions-title {
    font-size: 12px;
    color: var(--muted);
    margin-bottom: 6px;
}

.package-chip-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}

.package-chip {
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    border-radius: 999px;
    padding: 4px 10px;
    cursor: pointer;
    font-size: 12px;
}

.package-chip:hover {
    border-color: var(--accent);
    background: var(--panel-hover);
}

.validation-message {
    margin-top: 8px;
    font-size: 12px;
    color: var(--warning);
}

.wizard-footer {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    margin-top: 24px;
}

/* Language selector */
.language-selector {
    display: grid;
    gap: 12px;
}

.language-grid {
    display: grid;
    gap: 12px;
    grid-template-columns: 1fr;
}

.language-card {
    display: flex;
    gap: 12px;
    align-items: flex-start;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--surface);
    cursor: pointer;
    position: relative;
    transition: border-color 0.15s ease, background 0.15s ease;
}

.language-card:hover {
    border-color: var(--accent);
    background: var(--panel-hover);
}

.language-card.selected {
    border-color: var(--accent);
    background: rgba(88, 101, 242, 0.12);
}

.language-icon {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    background-size: cover;
    background-repeat: no-repeat;
    background-position: center;
    flex: 0 0 28px;
}

.language-info {
    display: grid;
    gap: 4px;
    flex: 1;
}

.language-name { font-weight: 600; }
.language-description { color: var(--muted); font-size: 12px; }
.language-versions { font-size: 12px; color: var(--text); }
.language-versions .version { color: var(--muted); }

.selected-indicator {
    position: absolute;
    top: 10px;
    right: 10px;
    width: 20px;
    height: 20px;
    border-radius: 999px;
    background: var(--accent);
    color: #fff;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
}

/* Logo marks */
.logo-python { background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'><rect width='64' height='64' rx='12' fill='%23306998'/><text x='32' y='40' font-size='26' text-anchor='middle' fill='white' font-family='Arial' font-weight='700'>Py</text></svg>"); }
.logo-node { background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'><polygon points='32,4 60,20 60,44 32,60 4,44 4,20' fill='%23339933'/><text x='32' y='40' font-size='22' text-anchor='middle' fill='white' font-family='Arial' font-weight='700'>JS</text></svg>"); }
.logo-rust { background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'><circle cx='32' cy='32' r='30' fill='%23f74c00'/><text x='32' y='40' font-size='22' text-anchor='middle' fill='white' font-family='Arial' font-weight='700'>Rs</text></svg>"); }
.logo-java { background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 64 64'><circle cx='32' cy='32' r='30' fill='%23f89820'/><text x='32' y='40' font-size='20' text-anchor='middle' fill='white' font-family='Arial' font-weight='700'>Java</text></svg>"); }

.modal h2 {
    margin: 0 0 20px;
    font-size: 20px;
    font-weight: 600;
}

.form-group {
    margin-bottom: 16px;
}

.form-group label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--text);
}

.form-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 24px;
    padding-top: 20px;
    border-top: 1px solid var(--border);
}

.form-input {
    width: 100%;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text);
    font-size: 14px;
}

.form-input:focus {
    outline: none;
    border-color: var(--accent);
    box-shadow: 0 0 0 2px rgba(88, 101, 242, 0.15);
}

.file-picker {
    display: flex;
    gap: 10px;
    align-items: center;
}

.file-picker-input {
    flex: 1;
}

.file-picker-button {
    white-space: nowrap;
}

.form-help {
    margin-top: 6px;
    font-size: 12px;
    color: var(--muted);
}

.modal-actions {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    margin-top: 24px;
}

/* Logs Modal */
.logs-modal {
    max-width: 900px;
    max-height: 80vh;
}

.logs-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
}

.logs-content {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 16px;
    max-height: 60vh;
    overflow: auto;
}

.logs-pre {
    margin: 0;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.5;
    color: var(--text);
    white-space: pre-wrap;
    word-wrap: break-word;
}

/* Dockerfile Editor Modal */
.dockerfile-editor-modal {
    max-width: 900px;
    width: 90%;
}

.dockerfile-path {
    margin-bottom: 16px;
    padding: 8px 12px;
    background: var(--panel-hover);
    border-radius: 6px;
    font-size: 13px;
    color: var(--muted);
}

.dockerfile-textarea {
    min-height: 400px;
    font-family: 'Courier New', monospace;
    font-size: 13px;
    line-height: 1.5;
}

/* Web Package Modal */
.web-package-modal {
    max-width: 1200px;
    width: 95%;
    max-height: 90vh;
    padding: 0;
    overflow: hidden;
}

.modal-large {
    max-width: 1200px;
    width: 95%;
}

.modal-tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    background: var(--panel-hover);
}

.tab-btn {
    flex: 1;
    padding: 12px 16px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    transition: all 0.15s;
    font-size: 14px;
    font-weight: 500;
}

.tab-btn:hover {
    background: var(--panel);
    color: var(--text);
}

.tab-btn.active {
    background: var(--surface);
    color: var(--accent);
    border-bottom: 2px solid var(--accent);
}

.modal-subtitle {
    font-size: 12px;
    color: var(--muted);
    margin-top: 4px;
}

.search-tab, .popular-tab, .categories-tab {
    padding: 20px;
    height: 60vh;
    overflow: auto;
}

.search-section {
    margin-bottom: 20px;
}

.search-bar {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
}

.search-input {
    flex: 1;
    padding: 10px 12px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
}

.search-btn {
    white-space: nowrap;
    padding: 10px 16px;
}

.search-filters {
    display: flex;
    gap: 12px;
    align-items: center;
}

.search-loading, .search-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--muted);
}

.search-error {
    padding: 12px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 6px;
    color: #ef4444;
    margin-bottom: 16px;
}

.placeholder-icon {
    font-size: 48px;
    margin-bottom: 12px;
}

.package-results {
    display: grid;
    gap: 12px;
}

.package-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    transition: all 0.15s;
}

.package-card:hover {
    border-color: var(--accent);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.package-card.selected {
    border-color: var(--accent);
    background: rgba(88, 101, 242, 0.05);
}

.package-header {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 16px;
}

.package-checkbox input {
    width: 16px;
    height: 16px;
    cursor: pointer;
}

.package-main-info {
    flex: 1;
}

.package-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 4px;
}

.package-name {
    font-weight: 600;
    color: var(--text);
    font-size: 16px;
}

.package-version {
    font-size: 12px;
    color: var(--muted);
    background: var(--panel-hover);
    padding: 2px 6px;
    border-radius: 4px;
}

.package-description {
    color: var(--muted);
    font-size: 14px;
    margin-bottom: 8px;
    line-height: 1.4;
}

.package-meta {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: var(--muted);
}

.package-actions {
    display: flex;
    gap: 4px;
}

.btn-icon {
    width: 28px;
    height: 28px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface);
    color: var(--muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
}

.btn-icon:hover {
    background: var(--panel-hover);
    color: var(--text);
}

.package-details {
    padding: 16px;
    border-top: 1px solid var(--border-light);
    background: var(--panel-hover);
}

.package-keywords {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 12px;
    flex-wrap: wrap;
}

.keywords-label {
    font-size: 12px;
    color: var(--muted);
    font-weight: 500;
}

.keyword-tag {
    font-size: 10px;
    background: var(--accent);
    color: white;
    padding: 2px 6px;
    border-radius: 4px;
}

.package-links {
    display: flex;
    gap: 12px;
}

.package-link {
    font-size: 12px;
    color: var(--accent);
    text-decoration: none;
}

.package-link:hover {
    text-decoration: underline;
}

.categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
}

.category-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    cursor: pointer;
    transition: all 0.15s;
    text-align: center;
}

.category-card:hover {
    border-color: var(--accent);
    background: var(--panel-hover);
}

.category-card.active {
    border-color: var(--accent);
    background: rgba(88, 101, 242, 0.05);
}

.category-icon {
    font-size: 32px;
    margin-bottom: 8px;
}

.category-name {
    font-weight: 600;
    margin-bottom: 4px;
}

.category-count {
    font-size: 12px;
    color: var(--muted);
}

.category-results {
    margin-top: 24px;
}

.tab-header {
    margin-bottom: 20px;
}

.tab-header h4 {
    margin-bottom: 4px;
}

.modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-top: 1px solid var(--border);
    background: var(--panel-hover);
}

.selected-info {
    font-size: 14px;
    color: var(--muted);
}

.selected-count {
    font-weight: 600;
    color: var(--accent);
}

.loading-spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border);
    border-top: 2px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 12px;
}

@keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}

.no-results {
    text-align: center;
    padding: 40px 20px;
    color: var(--muted);
}

.btn-success {
    background: var(--success);
    color: white;
    border: 1px solid var(--success);
}

.btn-success:hover {
    background: #157347;
    border-color: #157347;
}

.btn-success:disabled {
    background: var(--muted);
    border-color: var(--muted);
    cursor: not-allowed;
}

.form-select {
    padding: 6px 8px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface);
    color: var(--text);
    font-size: 12px;
}

/* Web Package Modal */
.web-package-modal {
    background: var(--surface);
    border-radius: 12px;
    max-width: 1200px;
    width: 95%;
    max-height: 90vh;
    padding: 0;
    overflow: hidden;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.35);
    display: flex;
    flex-direction: column;
}

.modal-large {
    max-width: 1200px;
    width: 95%;
    max-height: 90vh;
    padding: 0;
}

.modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-shrink: 0;
}

.modal-header h3 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: var(--text);
}

.modal-subtitle {
    font-size: 13px;
    color: var(--muted);
    margin-top: 4px;
}

.close-btn {
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    transition: all 0.15s;
    flex-shrink: 0;
}

.close-btn:hover {
    background: var(--panel-hover);
    border-color: var(--accent);
}

.modal-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
}

.modal-tabs {
    display: flex;
    border-bottom: 1px solid var(--border);
    background: var(--panel-hover);
}

.tab-btn {
    flex: 1;
    padding: 12px 16px;
    border: none;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    transition: all 0.15s;
    font-size: 14px;
    font-weight: 500;
}

.tab-btn:hover {
    background: rgba(77, 171, 247, 0.1);
    color: var(--text);
}

.tab-btn.active {
    background: var(--accent);
    color: white;
}

.search-tab, .popular-tab, .categories-tab {
    padding: 24px;
    height: 60vh;
    overflow-y: auto;
}

.search-section {
    margin-bottom: 24px;
}

.search-bar {
    display: flex;
    gap: 12px;
    margin-bottom: 16px;
}

.search-input {
    flex: 1;
    padding: 12px 16px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--surface);
    color: var(--text);
    font-size: 14px;
}

.search-btn {
    padding: 12px 20px;
    white-space: nowrap;
}

.search-filters {
    display: flex;
    gap: 12px;
    align-items: center;
}

.search-error {
    padding: 12px 16px;
    background: rgba(220, 53, 69, 0.1);
    border: 1px solid rgba(220, 53, 69, 0.3);
    border-radius: 6px;
    color: var(--danger);
    margin-bottom: 16px;
}

.search-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 40px;
    color: var(--muted);
    flex-direction: column;
    gap: 12px;
}

.loading-spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--border);
    border-top: 3px solid var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
}

@keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}

.no-results {
    text-align: center;
    padding: 40px;
    color: var(--muted);
}

.search-placeholder {
    text-align: center;
    padding: 60px 20px;
    color: var(--muted);
}

.placeholder-icon {
    font-size: 48px;
    margin-bottom: 16px;
}

.search-placeholder h4 {
    margin-bottom: 8px;
    color: var(--text);
}

.categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    margin-bottom: 24px;
}

.category-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 20px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    cursor: pointer;
    transition: all 0.15s;
    text-align: center;
}

.category-card:hover {
    border-color: var(--accent);
    background: var(--panel-hover);
}

.category-card.active {
    border-color: var(--accent);
    background: rgba(77, 171, 247, 0.1);
}

.category-icon {
    font-size: 32px;
    margin-bottom: 12px;
}

.category-name {
    font-weight: 500;
    margin-bottom: 4px;
    color: var(--text);
}

.category-count {
    font-size: 12px;
    color: var(--muted);
}

.package-results {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.package-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    transition: all 0.15s;
}

.package-card.selected {
    border-color: var(--accent);
    background: rgba(77, 171, 247, 0.05);
}

.package-header {
    display: flex;
    align-items: flex-start;
    padding: 16px;
    gap: 12px;
}

.package-checkbox {
    padding-top: 2px;
}

.package-main-info {
    flex: 1;
}

.package-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 8px;
}

.package-name {
    font-weight: 600;
    color: var(--text);
    font-size: 16px;
}

.package-version {
    padding: 2px 8px;
    background: var(--panel-hover);
    border-radius: 4px;
    font-size: 12px;
    color: var(--muted);
}

.package-description {
    margin-bottom: 8px;
    color: var(--muted);
    line-height: 1.4;
}

.package-meta {
    display: flex;
    gap: 16px;
    font-size: 12px;
    color: var(--muted);
}

.package-actions {
    display: flex;
    align-items: flex-start;
    padding-top: 2px;
}

.btn-icon {
    padding: 6px 8px;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--text);
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.15s;
}

.btn-icon:hover {
    background: var(--panel-hover);
    border-color: var(--accent);
}

.package-details {
    padding: 16px;
    border-top: 1px solid var(--border);
    background: var(--panel-hover);
}

.package-keywords {
    margin-bottom: 12px;
}

.keywords-label {
    font-weight: 500;
    margin-right: 8px;
    font-size: 12px;
    color: var(--muted);
}

.keyword-tag {
    display: inline-block;
    padding: 2px 6px;
    background: var(--accent);
    color: white;
    border-radius: 3px;
    font-size: 11px;
    margin-right: 4px;
}

.package-links {
    display: flex;
    gap: 12px;
}

.package-link {
    display: inline-flex;
    align-items: center;
    padding: 6px 12px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    text-decoration: none;
    color: var(--text);
    font-size: 12px;
    transition: all 0.15s;
}

.package-link:hover {
    border-color: var(--accent);
    color: var(--accent);
}

.modal-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 24px;
    border-top: 1px solid var(--border);
    background: var(--panel-hover);
}

.selected-info {
    font-size: 14px;
    color: var(--muted);
}

.selected-count {
    font-weight: 500;
    color: var(--text);
}

.tab-header {
    margin-bottom: 24px;
}

.tab-header h4 {
    margin-bottom: 4px;
    color: var(--text);
}

.tab-header p {
    color: var(--muted);
    font-size: 14px;
}

/* Installation Progress Styles */
.installation-progress {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.progress-header {
    text-align: center;
}

.progress-header h4 {
    margin: 0 0 8px 0;
    color: var(--text);
    font-size: 18px;
}

.progress-message {
    margin: 0;
    color: var(--accent);
    font-weight: 500;
    font-size: 14px;
}

.progress-packages {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px;
}

.progress-packages h5 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: var(--muted);
}

.package-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid var(--border);
}

.package-item:last-child {
    border-bottom: none;
}

.package-item.installing .package-status {
    color: var(--accent);
    animation: pulse 1.5s infinite;
}

.package-item.completed .package-status {
    font-weight: 500;
}

.package-name {
    font-weight: 500;
    color: var(--text);
}

.installation-complete {
    padding: 24px;
    text-align: center;
}

.success-message {
    color: var(--success);
}

.error-message {
    color: var(--danger);
}

.success-message h4, .error-message h4 {
    margin: 0 0 16px 0;
}

.installed-packages {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 16px;
    text-align: left;
    margin-top: 16px;
}

.installed-packages h5 {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: var(--muted);
}

@keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
}

/* Environment Details Modal */
.details-modal {
    max-width: 700px;
    width: 90%;
    max-height: 85vh;
}

.details-section {
    margin-bottom: 24px;
}

.details-section h4 {
    margin: 0 0 16px 0;
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
    border-bottom: 1px solid var(--border);
    padding-bottom: 8px;
}

.detail-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 16px;
}

.detail-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.detail-label {
    font-size: 12px;
    color: var(--muted);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}

.detail-value {
    font-size: 14px;
    color: var(--text);
}

.detail-value.path {
    font-family: 'Courier New', monospace;
    font-size: 12px;
    word-break: break-all;
}

.packages-section {
    margin-top: 24px;
}

.packages-list {
    max-height: 300px;
    overflow-y: auto;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px;
}

.package-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    border-bottom: 1px solid var(--border);
    transition: background 0.15s;
}

.package-item:last-child {
    border-bottom: none;
}

.package-item:hover {
    background: var(--panel-hover);
}

.package-item .package-name {
    font-weight: 500;
    color: var(--text);
    display: flex;
    align-items: center;
    gap: 8px;
}

.package-item .package-version {
    color: var(--muted);
    font-size: 13px;
    font-family: 'Courier New', monospace;
}

.dev-badge {
    background: var(--accent);
    color: white;
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
}

.empty-packages {
    padding: 32px;
    text-align: center;
    color: var(--muted);
    font-size: 14px;
}

/* Theme Toggle */
.theme-toggle-btn {
    width: 32px;
    height: 32px;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--panel);
    color: var(--text);
    cursor: pointer;
    font-size: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
    flex-shrink: 0;
}

.theme-toggle-btn:hover {
    background: var(--panel-hover);
    border-color: var(--accent);
}

/* Engine Manager */
.engine-info-panel {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 24px;
    margin-bottom: 32px;
    padding: 0 4px;
}

.info-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: transform 0.2s, box-shadow 0.2s;
}

.info-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.info-card h3 {
    margin: 0 0 20px 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.8px;
}

.info-row {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 12px 0;
    border-bottom: 1px solid var(--border-light);
    gap: 16px;
}

.info-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
}

.info-label {
    font-size: 14px;
    color: var(--muted);
    flex-shrink: 0;
    font-weight: 500;
}

.info-value {
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
    text-align: right;
    word-wrap: break-word;
    overflow-wrap: break-word;
    max-width: 100%;
}

.error-text {
    color: #ef4444;
    font-size: 13px;
    line-height: 1.5;
    word-break: break-word;
    white-space: pre-wrap;
}

.status-badge {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 6px 14px;
    border-radius: 16px;
    font-size: 13px;
    font-weight: 600;
}

.status-running {
    background: #10b98120;
    color: #10b981;
    border: 1px solid #10b98140;
}

.status-stopped {
    background: #ef444420;
    color: #ef4444;
    border: 1px solid #ef444440;
}

.logs-panel {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 24px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.logs-panel h3 {
    margin: 0 0 16px 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.8px;
}

.logs-container {
    background: var(--bg);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    max-height: 600px;
    overflow-y: auto;
    overflow-x: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--border) transparent;
}

.logs-container::-webkit-scrollbar {
    width: 8px;
    height: 8px;
}

.logs-container::-webkit-scrollbar-track {
    background: transparent;
}

.logs-container::-webkit-scrollbar-thumb {
    background: var(--border);
    border-radius: 4px;
}

.logs-container::-webkit-scrollbar-thumb:hover {
    background: var(--accent);
}

.logs-content {
    margin: 0;
    font-family: 'Fira Code', 'Monaco', 'Courier New', monospace;
    font-size: 13px;
    line-height: 1.7;
    color: var(--text);
    white-space: pre-wrap;
    word-wrap: break-word;
    word-break: break-word;
    overflow-wrap: break-word;
}

.logs-empty {
    text-align: center;
    padding: 60px 20px;
    color: var(--muted);
    font-size: 14px;
}

/* Toast Notifications */
.toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    z-index: 2000;
    pointer-events: none;
}

.toast {
    min-width: 320px;
    max-width: 420px;
    padding: 16px;
    border-radius: 8px;
    background: var(--surface);
    border: 1px solid var(--border);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    display: flex;
    align-items: flex-start;
    gap: 12px;
    pointer-events: auto;
    animation: slideInRight 0.3s ease-out;
}

@keyframes slideInRight {
    from {
        transform: translateX(100%);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}

.toast-icon {
    font-size: 20px;
    flex-shrink: 0;
}

.toast-content {
    flex: 1;
    font-size: 14px;
    line-height: 1.4;
}

.toast.toast-success {
    border-color: var(--success);
}

.toast.toast-success .toast-icon {
    color: var(--success);
}

.toast.toast-error {
    border-color: var(--danger);
}

.toast.toast-error .toast-icon {
    color: var(--danger);
}

.toast.toast-info {
    border-color: var(--accent);
}

.toast.toast-info .toast-icon {
    color: var(--accent);
}

/* Terminal styles */
.terminal-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
}

.terminal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    background: var(--panel);
    border-bottom: 1px solid var(--border);
}

.terminal-title {
    font-weight: 600;
    color: var(--text);
}

.terminal-info {
    display: flex;
    gap: 1rem;
    align-items: center;
    font-size: 0.875rem;
    color: var(--muted);
    flex: 1;
    justify-content: flex-end;
    margin-right: 12px;
}

.terminal-close-btn {
    width: 28px;
    height: 28px;
    border: 1px solid var(--border);
    border-radius: 4px;
    background: var(--surface);
    color: var(--text);
    cursor: pointer;
    font-size: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.15s;
    padding: 0;
    line-height: 1;
}

.terminal-close-btn:hover {
    background: var(--panel-hover);
    border-color: var(--accent);
    color: var(--accent);
}

.env-indicator {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    background: var(--border);
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 0.8rem;
}

.env-indicator.active {
    background: var(--success);
    color: #000;
}

.terminal-content {
    flex: 1;
    padding: 1rem;
    overflow-y: auto;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
    font-size: 0.875rem;
    line-height: 1.4;
    background: #0d1117;
    color: #c9d1d9;
    scroll-behavior: smooth;
    display: flex;
    flex-direction: column;
    overflow-anchor: none;
}

.terminal-scroll-anchor {
    overflow-anchor: auto;
    height: 1px;
    align-self: flex-end;
}

.terminal-line {
    display: flex;
    margin-bottom: 0.25rem;
    white-space: pre-wrap;
    word-break: break-all;
}

.terminal-line .timestamp {
    color: #6e7681;
    margin-right: 0.5rem;
    font-size: 0.8rem;
    min-width: 4rem;
}

.terminal-line .content {
    flex: 1;
}

.terminal-line.command .content {
    color: #58a6ff;
    font-weight: 500;
}

.terminal-line.output .content {
    color: #c9d1d9;
}

.terminal-line.error .content {
    color: #f85149;
}

.terminal-line.system .content {
    color: #a5a5a5;
    font-style: italic;
}

.terminal-input-line {
    display: flex;
    align-items: center;
    margin-top: 0.5rem;
    border-top: 1px solid var(--border);
    padding-top: 0.5rem;
}

.prompt {
    color: #58a6ff;
    margin-right: 0.5rem;
    font-weight: bold;
    white-space: nowrap;
}

.terminal-input {
    flex: 1;
    background: transparent;
    border: none;
    color: #c9d1d9;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
    font-size: 0.875rem;
    outline: none;
    padding: 0.25rem 0;
}

.terminal-input:focus {
    outline: none;
    box-shadow: none;
}

/* Layout with terminal styles */
.content-area {
    display: flex;
    flex-direction: column;
    height: 100vh;
}

.main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.main-content.with-terminal {
    flex: 1;
}

.terminal-panel {
    height: 300px;
    border-top: 1px solid var(--border);
    background: var(--surface);
    display: flex;
    flex-direction: column;
}

.section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 1.5rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
}

.section-header h1 {
    margin: 0;
    font-size: 1.5rem;
    font-weight: 600;
    color: var(--text);
}

.terminal-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.875rem;
}

.terminal-toggle.active {
    background: var(--accent);
    color: white;
}

.section-body {
    flex: 1;
    overflow: auto;
    padding: 1.5rem;
}

.main-content.with-terminal .section-body {
    padding-bottom: 1rem;
}
"#;

#[component]
pub fn AppRoot() -> Element {
    let mut app_state: Signal<AppState> = use_signal(AppState::default);
    use_context_provider(|| app_state);

    // Check if setup is completed
    let show_wizard = !app_state.read().setup.completed;

    if show_wizard {
        return rsx! {
            crate::components::setup::SetupWizard {
                on_complete: move |config| {
                    let mut state = app_state.write();
                    state.setup = config;
                }
            }
        };
    }

    let bridge = {
        let app_state = app_state.clone();
        use_coroutine(move |rx: UnboundedReceiver<BridgeAction>| async move {
            let mut app_state = app_state;
            let mut pending: VecDeque<Command> = VecDeque::new();
            let mut rx = rx.fuse();
            let addr = "ws://127.0.0.1:8765";

            'outer: loop {
                let connect = connect_async(addr).fuse();
                futures::pin_mut!(connect);

                let (mut sink, mut stream) = loop {
                    futures::select! {
                        conn = connect => {
                            match conn {
                                Ok((ws, _)) => {
                                    {
                                        let mut state = app_state.write();
                                        state.docker.connected = true;
                                        state.docker.last_error = None;
                                    }
                                    let (sink, stream) = ws.split();
                                    break (sink, stream.fuse());
                                }
                                Err(err) => {
                                    {
                                        let mut state = app_state.write();
                                        state.docker.connected = false;
                                        state.docker.last_error = Some(err.to_string());
                                    }
                                    tokio::time::sleep(Duration::from_secs(2)).await;
                                    continue 'outer;
                                }
                            }
                        }
                        action = rx.next() => {
                            match action {
                                Some(BridgeAction::SendCommand(cmd)) => pending.push_back(cmd),
                                None => return,
                            }
                        }
                    }
                };

                pending.push_back(Command::DockerList);
                pending.push_back(Command::DockerListImages);
                pending.push_back(Command::DockerListNetworks);
                pending.push_back(Command::DockerListVolumes);
                pending.push_back(Command::VirtEnvList); // Request environment list from backend

                while let Some(cmd) = pending.pop_front() {
                    if let Err(err) = send_command(&mut sink, cmd.clone()).await {
                        pending.push_front(cmd);
                        let mut state = app_state.write();
                        state.docker.connected = false;
                        state.docker.last_error = Some(err);
                        break;
                    }
                }

                loop {
                    futures::select! {
                        inbound = stream.next() => {
                            match inbound {
                                Some(Ok(Message::Text(text))) => {
                                    if let Err(err) = handle_event(app_state.clone(), &text) {
                                        let mut state = app_state.write();
                                        state.docker.last_error = Some(err);
                                    }
                                }
                                Some(Ok(Message::Close(_))) | None => {
                                    let mut state = app_state.write();
                                    state.docker.connected = false;
                                    break;
                                }
                                Some(Err(err)) => {
                                    let mut state = app_state.write();
                                    state.docker.connected = false;
                                    state.docker.last_error = Some(err.to_string());
                                    break;
                                }
                                _ => {}
                            }
                        }
                        action = rx.next() => {
                            match action {
                                Some(BridgeAction::SendCommand(cmd)) => {
                                    if let Err(err) = send_command(&mut sink, cmd.clone()).await {
                                        pending.push_back(cmd);
                                        let mut state = app_state.write();
                                        state.docker.connected = false;
                                        state.docker.last_error = Some(err);
                                        break;
                                    }
                                }
                                None => break,
                            }
                        }
                    }
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        })
    };
    
    // Provide bridge as context for components to send commands
    let backend_bridge = BackendBridge { tx: bridge };
    use_context_provider(|| backend_bridge);

    let bridge_handle = BackendBridge { tx: bridge };
    use_context_provider(|| bridge_handle);

    // Auto-dismiss toasts after 5 seconds
    use_effect(move || {
        let mut app_state = app_state;
        spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_millis(100)).await;
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                
                let mut state = app_state.write();
                state.ui.toasts.retain(|toast| {
                    now - toast.timestamp < 5000 // Keep toasts for 5 seconds
                });
            }
        });
    });

    let theme = app_state.read().user.theme.clone();
    let theme_css = match theme {
        Theme::Light => LIGHT_THEME,
        Theme::Dark => DARK_THEME,
    };
    
    let full_style = format!("{}{}", theme_css, BASE_STYLE);

    rsx! {
        style { {full_style} }
        AppRouter {}
        ToastContainer {}
        LogsModalComponent {}
        DockerfileEditorModal {}
        BuildConfirmationModal {}
    }
}

#[component]
fn ToastContainer() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let toasts = app_state.read().ui.toasts.clone();

    rsx! {
        div { class: "toast-container",
            for toast in toasts.iter() {
                ToastItem { key: "{toast.id}", toast: toast.clone() }
            }
        }
    }
}

#[component]
fn ToastItem(toast: crate::state::Toast) -> Element {
    let icon = match toast.toast_type {
        ToastType::Success => "✓",
        ToastType::Error => "✕",
        ToastType::Info => "ℹ",
    };
    
    let class_name = match toast.toast_type {
        ToastType::Success => "toast toast-success",
        ToastType::Error => "toast toast-error",
        ToastType::Info => "toast toast-info",
    };

    rsx! {
        div { class: "{class_name}",
            div { class: "toast-icon", "{icon}" }
            div { class: "toast-content", "{toast.message}" }
        }
    }
}

#[derive(Clone)]
pub struct BackendBridge {
    tx: Coroutine<BridgeAction>,
}

impl BackendBridge {
    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(BridgeAction::SendCommand(cmd));
    }
}

#[derive(Clone)]
enum BridgeAction {
    SendCommand(Command),
}

type WsSink = futures_util::stream::SplitSink<
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;

async fn send_command(
    sink: &mut WsSink,
    cmd: Command,
) -> Result<(), String> {
    let payload = serde_json::to_string(&cmd).map_err(|e| e.to_string())?;
    sink.send(Message::Text(payload)).await.map_err(|e| e.to_string())
}

fn handle_event(mut app_state: Signal<AppState>, payload: &str) -> Result<(), String> {
    match serde_json::from_str::<Event>(payload) {
        Ok(Event::DockerContainers(containers)) => {
            app_state.write().docker.containers = containers;
            Ok(())
        }
        Ok(Event::DockerStatus { connected, error }) => {
            let mut state = app_state.write();
            state.docker.connected = connected;
            state.docker.last_error = error;
            Ok(())
        }
        Ok(Event::DockerStats {
            containers,
            cpu_percent_avg,
            memory_used,
            memory_limit,
            net_rx,
            net_tx,
        }) => {
            let mut state = app_state.write();
            state.docker.stats.containers = containers;
            state.docker.stats.cpu_percent_avg = cpu_percent_avg;
            state.docker.stats.memory_used = memory_used;
            state.docker.stats.memory_limit = memory_limit;
            state.docker.stats.net_rx = net_rx;
            state.docker.stats.net_tx = net_tx;
            Ok(())
        }
        Ok(Event::DockerImages(images)) => {
            app_state.write().docker.images = images;
            Ok(())
        }
        Ok(Event::DockerNetworks(networks)) => {
            app_state.write().docker.networks = networks;
            Ok(())
        }
        Ok(Event::DockerVolumes(volumes)) => {
            app_state.write().docker.volumes = volumes;
            Ok(())
        }
        Ok(Event::DockerAction { action, ok, message }) => {
            let mut state = app_state.write();
            state.docker.action.in_progress = false;
            state.docker.action.last_action = Some(action.clone());
            state.docker.action.last_ok = Some(ok);
            state.docker.action.message = message.clone();
            
            // Add toast notification
            let toast_type = if ok { ToastType::Success } else { ToastType::Error };
            let toast_message = if ok {
                format!("{} completed successfully", action)
            } else {
                message.clone().unwrap_or_else(|| format!("{} failed", action))
            };
            
            let toast_id = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            
            state.ui.toasts.push(Toast {
                id: toast_id,
                message: toast_message,
                toast_type,
                timestamp: toast_id,
            });
            
            // Keep only last 5 toasts
            if state.ui.toasts.len() > 5 {
                state.ui.toasts.remove(0);
            }
            
            Ok(())
        }
        Ok(Event::DockerLogs { container_id, logs }) => {
            let mut state = app_state.write();
            state.ui.logs_modal = Some(LogsModal { container_id, logs });
            Ok(())
        }
        Ok(Event::DockerfileGenerated { path, dockerfile }) => {
            let mut state = app_state.write();
            state.ui.dockerfile_editor = Some(DockerfileEditor { path, dockerfile });
            Ok(())
        }
        Ok(Event::DockerfileSaved { path }) => {
            let mut state = app_state.write();
            state.ui.build_confirmation = Some(path);
            Ok(())
        }
        Ok(Event::DockerEngineLogs { logs }) => {
            let mut state = app_state.write();
            state.ui.engine_logs = Some(logs);
            Ok(())
        }
        Ok(Event::VirtualEnvSummary { total, active }) => {
            let mut state = app_state.write();
            state.virtenv.environments = total;
            state.virtenv.active = active;
            Ok(())
        }
        Ok(Event::VirtualEnvList(mut environments)) => {
            tracing::info!("Received VirtualEnvList event with {} environments from backend", environments.len());
            // Fix package counts if needed
            for env in &mut environments {
                if env.package_count == 0 && !env.packages.is_empty() {
                    env.package_count = env.packages.len();
                }
            }
            let mut state = app_state.write();
            state.virtenv.environment_list = environments;
            state.virtenv.environments = state.virtenv.environment_list.len();
            state.virtenv.active = state.virtenv.environment_list.iter().filter(|e| e.is_active).count();
            Ok(())
        }
        Ok(Event::VirtualEnvCreated { environment }) => {
            tracing::info!("🎉 Received VirtualEnvCreated event for environment: {} (id={})", environment.name, environment.id);
            let mut state = app_state.write();
            tracing::info!("📋 Before adding: {} environments in list", state.virtenv.environment_list.len());
            
            // Check if already exists
            if state.virtenv.environment_list.iter().any(|e| e.id == environment.id) {
                tracing::warn!("⚠️ Environment {} already exists in list! Not adding duplicate.", environment.id);
                return Ok(());
            }
            
            state.virtenv.environment_list.push(environment);
            state.virtenv.environments = state.virtenv.environment_list.len();
            state.virtenv.active = state.virtenv.environment_list.iter().filter(|e| e.is_active).count();
            tracing::info!("📋 After adding: {} environments in list", state.virtenv.environment_list.len());
            Ok(())
        }
        Ok(Event::VirtualEnvDeleted { env_id }) => {
            tracing::info!("Received VirtualEnvDeleted event for env_id: {}", env_id);
            let mut state = app_state.write();
            state.virtenv.environment_list.retain(|e| e.id != env_id);
            state.virtenv.environments = state.virtenv.environment_list.len();
            state.virtenv.active = state.virtenv.environment_list.iter().filter(|e| e.is_active).count();
            Ok(())
        }
        Ok(Event::PackageOperationCompleted { env_id, success, message }) => {
            tracing::info!("Received PackageOperationCompleted: env_id={}, success={}, message={:?}", env_id, success, message);
            let mut state = app_state.write();
            if let Some(op) = &mut state.virtenv.package_operation {
                if op.env_id == env_id {
                    op.in_progress = false;
                    op.success = Some(success);
                    op.message = message;
                }
            }
            Ok(())
        }
        Ok(Event::SystemProcessList(processes)) => {
            let mut state = app_state.write();
            state.system.processes = processes;
            Ok(())
        }
        Ok(Event::SystemAlert(alert)) => {
            let mut state = app_state.write();
            state.system.alerts.push(alert);
            if state.system.alerts.len() > 50 {
                state.system.alerts.remove(0);
            }
            Ok(())
        }
        Ok(Event::SystemSnapshot(snapshot)) => {
            apply_system_snapshot(app_state, snapshot);
            Ok(())
        }
        Ok(Event::NetworkTopology(topology)) => {
            let mut state = app_state.write();
            state.network.topology = Some(topology);
            state.network.topology_scanning = false;
            Ok(())
        }
        Ok(Event::RemoteDesktopConnectionsUpdated { connections }) => {
            let mut state = app_state.write();
            state.remote_desktop.connections = connections;
            Ok(())
        }
        Ok(Event::RemoteDesktopSessionsUpdated { sessions }) => {
            let mut state = app_state.write();
            state.remote_desktop.active_sessions = sessions;
            Ok(())
        }
        Err(err) => Err(format!("event parse failed: {err}")),
    }
}

fn apply_system_snapshot(mut app_state: Signal<AppState>, snapshot: SystemSnapshot) {
    let mut state = app_state.write();
    state.system.cpu_usage = snapshot.cpu_usage;
    state.system.memory_used = snapshot.memory_used;
    state.system.memory_total = snapshot.memory_total;
    state.system.swap_used = snapshot.swap_used;
    state.system.swap_total = snapshot.swap_total;
    state.system.uptime = snapshot.uptime;
    state.system.load_avg = Some(snapshot.load_avg);
    state.system.disks = snapshot.disks;
    state.system.networks = snapshot.networks;

    // Update history
    state.system.cpu_history.push(snapshot.cpu_usage);
    if state.system.cpu_history.len() > 60 {
        state.system.cpu_history.remove(0);
    }
    
    state.system.memory_history.push(snapshot.memory_used);
    if state.system.memory_history.len() > 60 {
        state.system.memory_history.remove(0);
    }
    
    // Calculate total network usage for this tick
    let (rx, tx) = state.system.networks.iter().fold((0, 0), |acc, n| (acc.0 + n.received, acc.1 + n.transmitted));
    
    state.system.network_rx_history.push(rx);
    if state.system.network_rx_history.len() > 60 {
        state.system.network_rx_history.remove(0);
    }
    
    state.system.network_tx_history.push(tx);
    if state.system.network_tx_history.len() > 60 {
        state.system.network_tx_history.remove(0);
    }
}

#[component]
pub fn ThemeToggle() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let theme = app_state.read().user.theme.clone();
    
    let toggle_theme = move |_| {
        let mut state = app_state.write();
        state.user.theme = match state.user.theme {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::Light,
        };
    };
    
    let icon = match theme {
        Theme::Light => "🌙",
        Theme::Dark => "☀️",
    };
    
    rsx! {
        button {
            class: "theme-toggle-btn",
            onclick: toggle_theme,
            title: "Toggle theme",
            "{icon}"
        }
    }
}

#[component]
fn LogsModalComponent() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let logs_modal = app_state.read().ui.logs_modal.clone();
    
    if let Some(modal) = logs_modal {
        let on_close = move |_| {
            app_state.write().ui.logs_modal = None;
        };
        
        rsx! {
            div { class: "modal-overlay", onclick: on_close,
                div { class: "modal logs-modal", onclick: move |e| e.stop_propagation(),
                    h2 { "Container Logs" }
                    div { class: "logs-header",
                        div { class: "cell-sub", "{modal.container_id[..12].to_string()}" }
                        button { class: "btn", onclick: on_close, "Close" }
                    }
                    div { class: "logs-content",
                        pre { class: "logs-pre", "{modal.logs}" }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}

#[component]
fn DockerfileEditorModal() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let editor = app_state.read().ui.dockerfile_editor.clone();
    
    if let Some(editor_data) = editor {
        let mut dockerfile_content = use_signal(|| editor_data.dockerfile.clone());
        let path_clone = editor_data.path.clone();
        
        let on_close = move |_| {
            app_state.write().ui.dockerfile_editor = None;
        };
        
        let on_save = move |_| {
            let content = dockerfile_content.read().clone();
            let path = path_clone.clone();
            
            bridge.send(Command::DockerSaveDockerfile {
                path,
                dockerfile: content,
            });
            
            app_state.write().ui.dockerfile_editor = None;
        };
        
        rsx! {
            div { class: "modal-overlay", onclick: on_close,
                div { class: "modal dockerfile-editor-modal", onclick: move |e| e.stop_propagation(),
                    h2 { "Edit Dockerfile" }
                    div { class: "dockerfile-path",
                        "Project: {editor_data.path}"
                    }
                    div { class: "form-group",
                        textarea {
                            class: "input dockerfile-textarea",
                            rows: "20",
                            value: "{dockerfile_content}",
                            oninput: move |e| dockerfile_content.set(e.value().clone()),
                            style: "font-family: 'Courier New', monospace; white-space: pre;"
                        }
                    }
                    div { class: "modal-actions",
                        button { class: "btn", onclick: on_close, "Cancel" }
                        button { class: "btn primary", onclick: on_save, "Save Dockerfile" }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}

#[component]
fn BuildConfirmationModal() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let build_path = app_state.read().ui.build_confirmation.clone();
    
    if let Some(path) = build_path {
        let path_clone = path.clone();
        
        let on_close = move |_| {
            app_state.write().ui.build_confirmation = None;
        };
        
        let on_build = move |_| {
            let tag = format!("{}:latest", path_clone.split('/').last().unwrap_or("app"));
            bridge.send(Command::DockerBuildImage {
                context_path: path_clone.clone(),
                tag: Some(tag),
            });
            app_state.write().ui.build_confirmation = None;
        };
        
        rsx! {
            div { class: "modal-overlay", onclick: on_close,
                div { class: "modal", onclick: move |e| e.stop_propagation(),
                    h2 { "Dockerfile Saved" }
                    p { "Dockerfile has been saved to:" }
                    div { class: "dockerfile-path",
                        "{path}/Dockerfile"
                    }
                    p { style: "margin-top: 16px;", "Would you like to build this as a Docker image?" }
                    div { class: "modal-actions",
                        button { class: "btn", onclick: on_close, "Not Now" }
                        button { class: "btn primary", onclick: on_build, "Build Image" }
                    }
                }
            }
        }
    } else {
        rsx! { div {} }
    }
}

