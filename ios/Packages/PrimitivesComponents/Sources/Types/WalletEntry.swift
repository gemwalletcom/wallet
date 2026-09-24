// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemWalletRow
import func Gemstone.walletRow
import GemstonePrimitives
import Primitives

public struct WalletEntry: Equatable, Sendable {
    public let wallet: Wallet
    public let row: GemWalletRow

    public init(wallet: Wallet, row: GemWalletRow) {
        self.wallet = wallet
        self.row = row
    }

    public init(wallet: Wallet) {
        self.init(wallet: wallet, row: walletRow(wallet: wallet.toGem()))
    }
}
