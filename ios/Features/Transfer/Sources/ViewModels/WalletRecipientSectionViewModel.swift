// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemAddressService
import struct Gemstone.GemRecipient
import Components
import Foundation
import GemstonePrimitives
import Primitives

struct WalletRecipientSectionViewModel {
    private let wallets: [Wallet]
    private let chain: Chain

    init(wallets: [Wallet], chain: Chain) {
        self.wallets = wallets
        self.chain = chain
    }

    var listItems: [ListItemValue<GemRecipient>] {
        let entries = wallets
            .compactMap { wallet -> (Wallet, Account)? in
                wallet.accounts.first { $0.chain == chain }.map { (wallet, $0) }
            }
        let subtitles = GemAddressService.shared.formatAll(
            addresses: entries.map { ChainAddress(chain: $0.1.chain, address: $0.1.address).toGem() },
            style: .short,
        )
        return zip(entries, subtitles).map { entry, subtitle in
            ListItemValue(
                title: entry.0.name,
                subtitle: subtitle,
                value: GemRecipient(address: entry.1.address, name: entry.0.name),
            )
        }
    }
}
