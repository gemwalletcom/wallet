// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol ExplorerLinkFetchable: Sendable {
    func addressUrl(chain: Chain, address: String) -> BlockExplorerLink
    func transactionUrl(chain: Chain, hash: String) -> BlockExplorerLink
    func swapTransactionUrl(chain: Chain, provider: String, input: ExplorerInput) -> BlockExplorerLink?
}

public extension ExplorerLinkFetchable {
    func transactionLink(
        chain: Chain,
        provider: String?,
        hash: String,
        recipient: String,
        memo: String?,
    ) -> BlockExplorerLink {
        guard let provider else {
            return transactionUrl(chain: chain, hash: hash)
        }
        let input = ExplorerInput(hash: hash, recipient: recipient, memo: memo)
        return swapTransactionUrl(chain: chain, provider: provider, input: input)
            ?? transactionUrl(chain: chain, hash: hash)
    }
}
