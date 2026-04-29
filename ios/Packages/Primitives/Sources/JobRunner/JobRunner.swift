// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

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

    static func getNextInterval(after current: Duration, config: JobConfiguration) -> Duration {
        max(config.initialInterval, min(current * Double(config.stepFactor), config.maxInterval))
    }
}

// MARK: - Private

extension JobRunner {
    private func removeJob(for id: String) {
        tasks.removeValue(forKey: id)
    }

    private func runJob(_ job: Job) async {
        var interval = job.configuration.initialInterval

        while !Task.isCancelled {
            let attemptStart = clock.now

            switch await job.run() {
            case .complete:
                do {
                    try await job.onComplete()
                } catch {
                    debugLog("job \(job.id) completed with error: \(error)")
                }
                return
            case .retry:
                let sleepUntil = attemptStart.advanced(by: interval)
                if clock.now < sleepUntil {
                    try? await clock.sleep(until: sleepUntil)
                }
                interval = Self.getNextInterval(after: interval, config: job.configuration)
            }
        }
    }
}
