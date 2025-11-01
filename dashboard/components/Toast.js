// Toast notification component
export class Toast {
    constructor(message, type = 'info', duration = 3000) {
        this.message = message;
        this.type = type; // success, error, warning, info
        this.duration = duration;
        this.id = 'toast-' + Date.now() + '-' + Math.random().toString(36).substr(2, 9);
    }

    show() {
        // Ensure toast container exists
        let container = document.getElementById('toast-container');
        if (!container) {
            container = document.createElement('div');
            container.id = 'toast-container';
            container.className = 'toast-container';
            document.body.appendChild(container);
        }

        const icons = {
            success: 'fa-check-circle',
            error: 'fa-exclamation-circle',
            warning: 'fa-exclamation-triangle',
            info: 'fa-info-circle'
        };

        const toast = document.createElement('div');
        toast.id = this.id;
        toast.className = `toast toast-${this.type}`;
        toast.innerHTML = `
            <div class="toast-content">
                <i class="fas ${icons[this.type] || icons.info}"></i>
                <span>${this.message}</span>
            </div>
            <button class="toast-close" onclick="this.parentElement.remove()">&times;</button>
        `;

        container.appendChild(toast);

        // Trigger animation
        setTimeout(() => {
            toast.classList.add('show');
        }, 10);

        // Auto remove
        if (this.duration > 0) {
            setTimeout(() => {
                this.remove();
            }, this.duration);
        }

        return this;
    }

    remove() {
        const toast = document.getElementById(this.id);
        if (toast) {
            toast.classList.remove('show');
            setTimeout(() => {
                toast.remove();
            }, 300);
        }
    }
}

// Helper functions
export const toast = {
    success(message, duration = 3000) {
        return new Toast(message, 'success', duration).show();
    },
    
    error(message, duration = 5000) {
        return new Toast(message, 'error', duration).show();
    },
    
    warning(message, duration = 4000) {
        return new Toast(message, 'warning', duration).show();
    },
    
    info(message, duration = 3000) {
        return new Toast(message, 'info', duration).show();
    }
};

// Make toast globally available
window.toast = toast;

