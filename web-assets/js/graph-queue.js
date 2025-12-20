// =====================================================
// Graph Operation Queue - Prevents Animation Conflicts
// =====================================================

/**
 * A queue for graph operations to prevent animation conflicts.
 * Operations are executed sequentially, with each waiting for the previous to complete.
 */
class GraphOperationQueue {
    constructor() {
        this.queue = [];
        this.isProcessing = false;
        this.currentOperation = null;
    }

    /**
     * Add an operation to the queue
     * @param {string} name - Operation name for debugging
     * @param {Function} operation - Async function to execute
     * @param {Object} options - Options for the operation
     * @param {boolean} options.cancelPending - Cancel all pending operations of the same name
     * @param {number} options.timeout - Max time to wait for operation (default: 5000ms)
     * @returns {Promise} - Resolves when operation completes
     */
    enqueue(name, operation, options = {}) {
        const { cancelPending = false, timeout = 5000 } = options;

        // Cancel pending operations with same name if requested
        if (cancelPending) {
            this.queue = this.queue.filter(op => op.name !== name);
        }

        return new Promise((resolve, reject) => {
            this.queue.push({
                name,
                operation,
                timeout,
                resolve,
                reject,
                timestamp: Date.now()
            });

            this._processNext();
        });
    }

    /**
     * Process the next operation in the queue
     */
    async _processNext() {
        if (this.isProcessing || this.queue.length === 0) {
            return;
        }

        this.isProcessing = true;
        this.currentOperation = this.queue.shift();

        const { name, operation, timeout, resolve, reject } = this.currentOperation;

        try {
            // Create a timeout promise
            const timeoutPromise = new Promise((_, timeoutReject) => {
                setTimeout(() => {
                    timeoutReject(new Error(`Operation '${name}' timed out after ${timeout}ms`));
                }, timeout);
            });

            // Race between operation and timeout
            const result = await Promise.race([
                Promise.resolve(operation()),
                timeoutPromise
            ]);

            resolve(result);
        } catch (error) {
            console.warn(`Graph operation '${name}' failed:`, error);
            reject(error);
        } finally {
            this.isProcessing = false;
            this.currentOperation = null;

            // Process next operation
            if (this.queue.length > 0) {
                // Small delay to prevent UI blocking
                requestAnimationFrame(() => this._processNext());
            }
        }
    }

    /**
     * Clear all pending operations
     */
    clear() {
        this.queue.forEach(op => {
            op.reject(new Error('Operation cancelled'));
        });
        this.queue = [];
    }

    /**
     * Get queue status
     */
    getStatus() {
        return {
            pending: this.queue.length,
            processing: this.isProcessing,
            currentOperation: this.currentOperation?.name || null
        };
    }
}

// Singleton instance
export const graphQueue = new GraphOperationQueue();

/**
 * Wait for layout animation to complete
 * @param {Object} cy - Cytoscape instance
 * @param {Object} layoutConfig - Layout configuration
 * @returns {Promise} - Resolves when layout is complete
 */
export function runLayoutAsync(cy, layoutConfig) {
    return new Promise((resolve) => {
        if (!cy) {
            resolve();
            return;
        }

        const layout = cy.layout({
            ...layoutConfig,
            ready: () => {},
            stop: () => resolve()
        });

        layout.run();

        // Fallback timeout in case layout doesn't fire stop event
        const animDuration = layoutConfig.animationDuration || 500;
        setTimeout(resolve, animDuration + 100);
    });
}

/**
 * Debounce helper
 */
export function debounce(fn, delay) {
    let timeoutId;
    return function (...args) {
        clearTimeout(timeoutId);
        timeoutId = setTimeout(() => fn.apply(this, args), delay);
    };
}
