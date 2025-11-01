// Utility functions for formatting data
const formatters = {
    formatNumber(num) {
        if (num === null || num === undefined) return '0';
        return new Intl.NumberFormat('pt-BR').format(num);
    },

    formatDate(dateString) {
        if (!dateString) return 'N/A';
        return new Date(dateString).toLocaleDateString('pt-BR', {
            year: 'numeric',
            month: 'short',
            day: 'numeric'
        });
    },

    formatDateTime(dateString) {
        if (!dateString) return 'N/A';
        return new Date(dateString).toLocaleString('pt-BR', {
            year: 'numeric',
            month: 'short',
            day: 'numeric',
            hour: '2-digit',
            minute: '2-digit'
        });
    },

    formatStatus(status) {
        const statusMap = {
            completed: 'Concluído',
            processing: 'Processando',
            indexing: 'Indexando',
            pending: 'Pendente',
            failed: 'Falhou',
            cached: 'Do Cache'
        };
        return statusMap[status] || status;
    },

    formatSize(bytes) {
        if (!bytes || bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
    },

    truncateText(text, maxLength) {
        if (!text) return '';
        if (text.length <= maxLength) return text;
        return text.substring(0, maxLength) + '...';
    },

    formatJSON(obj) {
        if (!obj) return 'null';
        try {
            const jsonStr = JSON.stringify(obj, null, 2);
            // Simple syntax highlighting
            return jsonStr
                .replace(/"([^"]+)":/g, '<span class="json-key">"$1":</span>')
                .replace(/: "([^"]*)"/g, ': <span class="json-string">"$1"</span>')
                .replace(/: (\d+\.?\d*)/g, ': <span class="json-number">$1</span>')
                .replace(/: (true|false)/g, ': <span class="json-boolean">$1</span>')
                .replace(/: null/g, ': <span class="json-null">null</span>');
        } catch (e) {
            return String(obj);
        }
    },

    formatDuration(seconds) {
        if (!seconds || seconds === 0) return '0s';
        const units = [
            { value: 31536000, label: 'y' },
            { value: 86400, label: 'd' },
            { value: 3600, label: 'h' },
            { value: 60, label: 'm' },
            { value: 1, label: 's' }
        ];
        
        let remaining = seconds;
        const parts = [];
        
        for (const unit of units) {
            const count = Math.floor(remaining / unit.value);
            if (count > 0) {
                parts.push(`${count}${unit.label}`);
                remaining %= unit.value;
            }
        }
        
        return parts.join(' ') || '0s';
    }
};

// Make formatters globally available
window.formatters = formatters;

