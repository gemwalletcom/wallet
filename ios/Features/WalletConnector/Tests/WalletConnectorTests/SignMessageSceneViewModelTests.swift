// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemSimulationValue
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Foundation
import struct Gemstone.SimulationHeader
import func Gemstone.walletRow
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing
@testable import WalletConnector
import WalletConnectorTestKit
import WalletConnectorService
import WalletConnectorServiceTestKit

struct SignMessageSceneViewModelTests {
    @Test
    @MainActor
    func walletTextDisplaysPayloadWallet() {
        let wallet = Wallet.mock(name: "My Secure Wallet")
        let payload = SignMessagePayload.mock(wallet: wallet)

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.walletText == "My Secure Wallet")
    }

    @Test
    @MainActor
    func appTextUsesShortNameWithoutDomain() {
        let payload = SignMessagePayload.mock(
            session: .mock(metadata: .mock(
                name: "PancakeSwap - Trade",
                url: "https://pancakeswap.finance/swap",
            )),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.appText == "PancakeSwap")
    }

    @Test
    @MainActor
    func titleUsesReviewRequest() {
        let viewModel = SignMessageSceneViewModel.mock()

        #expect(viewModel.title == "Review Request")
    }

    @Test
    @MainActor
    func payloadStoresValidatedChainNotMessageChain() {
        let payload = SignMessagePayload.mock(message: .mock(chain: "bitcoin"))

        #expect(payload.chain == .ethereum)
        #expect(payload.message.chain == "bitcoin")
    }

    @Test
    @MainActor
    func networkTextUsesPayloadChain() {
        let payload = SignMessagePayload.mock(message: .mock(chain: "bitcoin"))

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.networkText == "Ethereum")
    }

    @Test
    @MainActor
    func contextRowsProvideWalletAndNetworkImages() {
        let payload = SignMessagePayload.mock()

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.walletAssetImage == walletRow(wallet: payload.wallet.toGem()).avatarImage)
        #expect(viewModel.networkAssetImage == AssetIdViewModel(assetId: payload.chain.asset.id).networkAssetImage)
    }

    @Test
    @MainActor
    func buttonEnabledWithNoWarnings() {
        let viewModel = SignMessageSceneViewModel.mock()

        #expect(!viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func buttonEnabledWithNonCriticalWarnings() {
        let payload = SignMessagePayload.mock(
            simulation: .mock(warnings: [.mock()]),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(!viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func simulationWarningsHideBoundedApprovals() {
        let payload = SignMessagePayload.mock(
            simulation: .mock(warnings: [
                .mock(warning: .tokenApproval(.mock(value: 1000))),
                .mock(),
            ]),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.simulationWarnings.map(\.kind) == [.unlimitedApproval])
    }

    @Test
    @MainActor
    func buttonDisabledWithCriticalWarnings() {
        let payload = SignMessagePayload.mock(
            simulation: .mock(warnings: [.mock(severity: .critical, warning: .suspiciousSpender)]),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func simulationWarningsHideBoundedApprovalsAndKeepExternallyOwnedSpenderWarnings() {
        let payload = SignMessagePayload.mock(
            simulation: .mock(warnings: [
                .mock(warning: .permitApproval(.mock(value: 1000))),
                .mock(warning: .externallyOwnedSpender),
            ]),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.simulationWarnings.map(\.kind) == [.externallyOwnedSpender])
        #expect(!viewModel.isButtonDisabled)
    }

    @Test
    @MainActor
    func permitBatchExternallyOwnedSpenderKeepsWarningAndPayload() {
        let payload = SignMessagePayload.mock(
            message: .mockPermitBatch(),
            simulation: .mockPermitBatch(warnings: [.mock(warning: .externallyOwnedSpender)]),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.simulationWarnings.count == 1)
        #expect(viewModel.simulationWarnings.first?.kind == .externallyOwnedSpender)
        #expect(!viewModel.isButtonDisabled)
        #expect(viewModel.payloadModel.hasFields)
        #expect(viewModel.payloadModel.primaryFields.contains(where: { $0.kind == .spender && $0.value == "0x3333333333333333333333333333333333333333" }))
    }

    @Test
    @MainActor
    func simulationWarningsPassThroughValidationWarnings() {
        let payload = SignMessagePayload.mock(
            simulation: .mock(warnings: [
                .mock(warning: .permitApproval(.mock(value: 1000))),
                .mock(severity: .critical, warning: .validationError, message: "Unable to verify spender is a contract"),
            ]),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.simulationWarnings.map(\.kind) == [.validationError])
    }

    @Test
    @MainActor
    func siweChainMismatchStillUsesStructuredPayload() {
        let message = [
            "thepoc.xyz wants you to sign in with your Ethereum account:",
            "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4",
            "",
            "Sign in with different chain ID",
            "",
            "URI: https://thepoc.xyz",
            "Version: 1",
            "Chain ID: 137",
            "Nonce: gv7zples2q60kq7bnamtuwo",
            "Issued At: 2026-03-11T04:20:21.742Z",
        ]
        .joined(separator: "\n")

        let payload = SignMessagePayload.mock(
            message: .mock(data: Data(message.utf8)),
            simulation: .mock(warnings: [
                .mock(severity: .critical, warning: .validationError, message: "Chain ID mismatch"),
            ]),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.title == "Sign In with Ethereum")
        #expect(viewModel.isButtonDisabled)
        #expect(viewModel.payloadModel.hasFields)
        #expect(viewModel.payloadModel.primaryFields.count == 2)
    }

    @Test
    @MainActor
    func permitHeaderReplacesValueField() {
        let asset = Asset.mockEthereumUSDT()
        let payload = SignMessagePayload.mock(
            message: .mockPermitBatch(),
            simulation: .mockPermitBatch(header: SimulationHeader(assetId: asset.id.identifier, value: nil, isUnlimited: true)),
            assets: [asset],
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.headerData == GemSimulationValue(asset: asset.toGem(), value: .unlimited))
        #expect(!(viewModel.payloadModel.primaryFields + viewModel.payloadModel.secondaryFields).contains { $0.kind == .value })
    }

    @Test
    @MainActor
    func permitWithoutHeaderKeepsValueField() {
        let payload = SignMessagePayload.mock(
            message: .mockPermitBatch(),
            simulation: .mockPermitBatch(),
        )

        let viewModel = SignMessageSceneViewModel.mock(payload: payload)

        #expect(viewModel.headerData == nil)
        #expect((viewModel.payloadModel.primaryFields + viewModel.payloadModel.secondaryFields).contains { $0.kind == .value })
    }
}
