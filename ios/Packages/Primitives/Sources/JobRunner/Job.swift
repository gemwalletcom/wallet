// Copyright (c). Gem Wallet. All rights reserved.

public protocol Job: Sendable {
    var id: String { get }
    var configuration: JobConfiguration { get }

    func run() async -> JobStatus
    func nextInterval(after currentIntervalMs: UInt32) -> UInt32
    func onComplete() async throws
}
