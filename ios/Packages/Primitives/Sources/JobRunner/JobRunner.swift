// Copyright (c). Gem Wallet. All rights reserved.

public actor JobRunner {
    private var tasks: [String: Task<Void, Never>] = [:]
    private let clock: ContinuousClock

    public init(clock: ContinuousClock = ContinuousClock()) {
        self.clock = clock
    }

    public func addJob(job: Job) {
        tasks[job.id]?.cancel()
        tasks[job.id] = Task { [weak self] in
            await self?.runJob(job)
            await self?.removeJob(for: job.id)
        }
    }

    public func cancelJob(id: String) {
        tasks[id]?.cancel()
        removeJob(for: id)
    }

    public func stopAll() {
        tasks.keys.forEach(cancelJob)
    }
}

// MARK: - Private

extension JobRunner {
    private func removeJob(for id: String) {
        tasks.removeValue(forKey: id)
    }

    private func runJob(_ job: Job) async {
        var intervalMs = job.configuration.initialIntervalMs

        while !Task.isCancelled {
            let attemptStart = clock.now

            switch await job.run() {
            case .complete:
                do {
                    try await job.onComplete()
                    debugLog("transaction status complete: id=\(job.id), status=complete")
                } catch {
                    debugLog("transaction status complete: id=\(job.id), status=complete, error=\(error)")
                }
                return
            case let .retry(error):
                let sleepUntil = attemptStart.advanced(by: .milliseconds(Int(intervalMs)))
                if clock.now < sleepUntil {
                    try? await clock.sleep(until: sleepUntil)
                }
                intervalMs = job.nextInterval(after: intervalMs)
                if let error {
                    debugLog("transaction status pending: id=\(job.id), next check = \(intervalMs)ms, error=\(error)")
                } else {
                    debugLog("transaction status pending: id=\(job.id), next check = \(intervalMs)ms")
                }
            }
        }
    }
}
