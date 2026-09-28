// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import os

extension URLSessionWebSocketTask {
    func sendPing() async throws {
        let resumed = OSAllocatedUnfairLock(initialState: false)
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, any Error>) in
            sendPing { error in
                let isFirst = resumed.withLock { resumed in
                    guard !resumed else { return false }
                    resumed = true
                    return true
                }
                guard isFirst else { return }
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume()
                }
            }
        }
    }
}
