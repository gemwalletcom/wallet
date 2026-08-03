// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public enum SignableTransaction: Sendable {
    case ethereum(EthereumTransactionData, TransactionType)
    case solana(String, TransferDataOutputType)
    case sui(String, TransferDataOutputType)
    case ton(String, TransferDataOutputType)
    case tron(String, TransferDataOutputType)
}
