// Reusable StatCard component
class StatCard {
    constructor(config) {
        this.icon = config.icon;
        this.value = config.value;
        this.label = config.label;
        this.subtitle = config.subtitle;
        this.variant = config.variant || 'primary';
        this.onClick = config.onClick;
    }

    render() {
        const variantClasses = {
            primary: 'bg-blue-500/20 text-blue-400',
            success: 'bg-green-500/20 text-green-400',
            warning: 'bg-yellow-500/20 text-yellow-400',
            error: 'bg-red-500/20 text-red-400',
            info: 'bg-cyan-500/20 text-cyan-400',
            secondary: 'bg-bg-tertiary text-text-secondary'
        };

        const iconClass = variantClasses[this.variant] || variantClasses.primary;
        const clickable = this.onClick ? 'cursor-pointer hover:bg-bg-hover hover:border-border-light transition-all hover:-translate-y-0.5' : '';

        return `
            <div class="stat-card ${clickable}" ${this.onClick ? `onclick="${this.onClick}"` : ''}>
                <div class="stat-icon ${iconClass}">
                    <i class="${this.icon}"></i>
                </div>
                <div class="stat-content">
                    <div class="stat-value">${this.value}</div>
                    <div class="stat-label">${this.label}</div>
                    ${this.subtitle ? `<div class="stat-subtitle">${this.subtitle}</div>` : ''}
                </div>
            </div>
        `;
    }
}

// Helper function to create stat cards
function createStatCard(config) {
    const card = new StatCard(config);
    return card.render();
}

// Make StatCard globally available
window.StatCard = StatCard;
window.createStatCard = createStatCard;

