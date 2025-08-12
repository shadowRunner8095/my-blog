export class StatefulObserver<S> {
    private state: S;
    private subscribers: Set<(newState: S) => void> = new Set();

    constructor(initialState: S) {
        this.state = initialState;
    }

    public getState(): S {
        return this.state;
    }

    public subscribe(callback: (newState: S) => void): () => void {
        this.subscribers.add(callback);
        // Immediately notify the new subscriber with the current state.
        callback(this.state);
        return () => this.unsubscribe(callback);
    }

    public unsubscribe(callback: (newState: S) => void) {
        this.subscribers.delete(callback);
    }

    public notify(newState: S) {
        if (newState === this.state) {
            return;
        }
        this.state = newState;
        this.subscribers.forEach(callback => callback(this.state));
    }
}
