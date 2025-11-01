// Modal component for reusable modals
class Modal {
    constructor(options = {}) {
        this.id = options.id || 'modal-' + Date.now();
        this.title = options.title || '';
        this.content = options.content || '';
        this.footer = options.footer || '';
        this.size = options.size || 'md'; // sm, md, lg, xl
        this.closeOnBackdrop = options.closeOnBackdrop !== false;
        this.onClose = options.onClose;
    }

    show() {
        const modalHTML = `
            <div id="${this.id}" class="modal-overlay" ${this.closeOnBackdrop ? 'onclick="if(event.target === this) window.modal.close()"' : ''}>
                <div class="modal-container modal-${this.size}">
                    ${this.title ? `
                        <div class="modal-header">
                            <h2>${this.title}</h2>
                            <button class="modal-close" onclick="window.modal.close()">&times;</button>
                        </div>
                    ` : ''}
                    <div class="modal-body">
                        ${this.content}
                    </div>
                    ${this.footer ? `
                        <div class="modal-footer">
                            ${this.footer}
                        </div>
                    ` : ''}
                </div>
            </div>
        `;

        // Create modal container if it doesn't exist
        let modalContainer = document.getElementById('modal-container');
        if (!modalContainer) {
            modalContainer = document.createElement('div');
            modalContainer.id = 'modal-container';
            document.body.appendChild(modalContainer);
        }

        modalContainer.innerHTML = modalHTML;

        // Add escape key listener
        const escapeHandler = (e) => {
            if (e.key === 'Escape') {
                this.close();
                document.removeEventListener('keydown', escapeHandler);
            }
        };
        document.addEventListener('keydown', escapeHandler);

        // Store escape handler for cleanup
        this._escapeHandler = escapeHandler;

        // Trigger animation
        setTimeout(() => {
            const overlay = document.getElementById(this.id);
            if (overlay) {
                overlay.classList.add('show');
            }
        }, 10);
    }

    close() {
        const modal = document.getElementById(this.id);
        if (modal) {
            modal.classList.remove('show');
            setTimeout(() => {
                if (this.onClose) {
                    this.onClose();
                }
                modal.remove();
                if (this._escapeHandler) {
                    document.removeEventListener('keydown', this._escapeHandler);
                }
            }, 300);
        }
    }

    updateContent(content) {
        const body = document.querySelector(`#${this.id} .modal-body`);
        if (body) {
            body.innerHTML = content;
        }
    }
}

// Global modal instance
window.modal = {
    current: null,
    
    show(options) {
        if (this.current) {
            this.current.close();
        }
        this.current = new Modal(options);
        this.current.show();
        return this.current;
    },
    
    close() {
        if (this.current) {
            this.current.close();
            this.current = null;
        }
    }
};

