// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemTransferData
import Primitives
import WalletConnector

public struct ConfirmTransferRequest: Sendable {
    public let data: GemTransferData
    public let delegate: TransferDataCallback.ConfirmTransferDelegate?

    public init(
        data: GemTransferData,
        delegate: TransferDataCallback.ConfirmTransferDelegate? = nil,
    ) {
        self.data = data
        self.delegate = delegate
    }
}
