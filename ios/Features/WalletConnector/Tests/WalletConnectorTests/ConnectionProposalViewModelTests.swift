// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing
@testable import WalletConnector

struct ConnectionProposalViewModelTests {
    @Test
    func appIconUsesProxy() {
        let metadata = ApplicationMetadata.mock(url: "https://app.example.com/swap/", icon: "../icon.svg?v=1&theme=dark")
        let model = ConnectionProposalViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: metadata)),
        )

        #expect(model.imageUrl?.absoluteString == "https://assets.gemwallet.com/proxy/image?url=https%3A%2F%2Fapp.example.com%2Ficon.svg%3Fv%3D1%26theme%3Ddark&size=256")
        #expect(model.websiteText == "app.example.com")
    }

    @Test
    func unsafeAppIconUsesPlaceholder() {
        let model = ConnectionProposalViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: .mock(icon: "http://app.example.com/icon.png"))),
        )

        #expect(model.imageUrl == nil)
    }

    @Test
    func appTextKeepsNameAndDomain() {
        let metadata = ApplicationMetadata.mock(
            name: "PancakeSwap - Trade",
            url: "https://pancakeswap.finance/swap",
        )
        let model = ConnectionProposalViewModel(
            confirmTransferDelegate: { _ in },
            pairingProposal: .mock(proposal: .mock(metadata: metadata)),
        )

        #expect(model.appText == "PancakeSwap (pancakeswap.finance)")
    }
}
