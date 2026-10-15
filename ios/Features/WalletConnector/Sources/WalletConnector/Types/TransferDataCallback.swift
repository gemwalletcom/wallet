// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public final class TransferDataCallback<T: Sendable>: Sendable {
    public typealias ConfirmTransferDelegate = @Sendable (Result<String, any Error>) -> Void

    public let payload: T
    public let delegate: ConfirmTransferDelegate

    public init(
        payload: T,
        delegate: @escaping ConfirmTransferDelegate,
    ) {
        self.payload = payload
        self.delegate = delegate
    }
}
