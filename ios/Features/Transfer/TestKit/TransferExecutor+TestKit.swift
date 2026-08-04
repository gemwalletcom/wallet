// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Transfer

public struct TransferExecutorMock: TransferExecutable {
    public var error: Error?
    public var results: [String]

    public init(error: Error? = nil, results: [String] = ["1"]) {
        self.error = error
        self.results = results
    }

    @discardableResult
    public func execute(input _: TransferConfirmationInput) async throws -> [String] {
        if let error {
            throw error
        }
        return results
    }
}
