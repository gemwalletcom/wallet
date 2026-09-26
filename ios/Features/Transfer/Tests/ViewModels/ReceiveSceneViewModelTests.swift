// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Gemstone
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Localization
import Primitives
import PrimitivesTestKit
import SwiftUI
import Testing
@testable import Transfer
import TransferTestKit

@MainActor
struct ReceiveSceneViewModelTests {
    private let bitcoin = Primitives.Asset.mock(id: .mock(chain: .bitcoin))
    private let ethereum = Primitives.Asset.mock(id: .mock(chain: .ethereum))
    private let solana = Primitives.Asset.mock(id: .mock(chain: .solana))

    @Test
    func theNetworksComeFromCore() {
        let service = GemReceiveServiceMock()
        service.networksValue = GemReceiveNetworks(
            networks: [
                GemReceiveNetwork(assetId: bitcoin.id.identifier, standard: nil),
                GemReceiveNetwork(assetId: ethereum.id.identifier, standard: .text(text: "ERC20")),
            ],
            showsSelector: true,
        )
        let model = ReceiveSceneViewModel.mock(service: service)

        #expect(model.networkSelectorModel.items == [bitcoin.id, ethereum.id])
        #expect(model.showNetworkSelector)
        #expect(model.chainModel(for: ethereum.id).listItem.titleExtra == "ERC20")
        #expect(model.chainModel(for: bitcoin.id).listItem.titleExtra == nil, "a coin names no token standard")
    }

    @Test
    func oneNetworkHidesTheSelector() {
        let model = ReceiveSceneViewModel.mock()

        #expect(model.showNetworkSelector == false)
    }

    @Test
    func theWarningsComeFromCore() {
        let service = GemReceiveServiceMock()
        service.warningsValue = [.noMemoRequired, .noDestinationTagRequired]
        let model = ReceiveSceneViewModel.mock(service: service)

        #expect(model.warningMessage.contains(Localized.Wallet.Receive.noMemoRequired))
        #expect(model.warningMessage.contains(Localized.Wallet.Receive.noDestinationTagRequired))
    }

    @Test
    func noWarningsMeanNoMessage() {
        let model = ReceiveSceneViewModel.mock()

        #expect(model.warningMessage.isEmpty)
    }

    @Test
    func showingTheSceneEnablesTheAssetItShows() async {
        let service = GemReceiveServiceMock()
        let model = ReceiveSceneViewModel.mock(service: service)

        await model.onChangeAsset()

        #expect(service.enabledAssetIds == [bitcoin.id.identifier])
    }

    @Test
    func aFailedEnableLeavesNoAlert() async {
        let service = GemReceiveServiceMock()
        service.enableAssetError = AnyError("offline")
        let model = ReceiveSceneViewModel.mock(service: service)

        await model.onChangeAsset()

        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func pickingTheSameNetworkChangesNothing() async {
        let service = GemReceiveServiceMock()
        let model = ReceiveSceneViewModel.mock(service: service)
        let address = model.copyModel.content.display

        model.onSelectNetwork()
        #expect(model.presentation == .networkSelector)

        model.onFinishNetworkSelection([bitcoin.id])
        await model.selectNetworkTask?.value

        #expect(model.presentation == nil)
        #expect(model.copyModel.content.display == address)
        #expect(service.requestedAssetIds.isEmpty)
    }

    @Test
    func pickingAnotherNetworkSwapsTheAssetAndItsAddress() async {
        let service = GemReceiveServiceMock()
        service.assetResult = .success(ethereum.toGem())
        let model = ReceiveSceneViewModel.mock(service: service)

        model.onFinishNetworkSelection([ethereum.id])
        await model.selectNetworkTask?.value
        await model.onChangeAsset()

        #expect(service.requestedAssetIds == [ethereum.id.identifier])
        #expect(model.asset.chain == .ethereum)
        #expect(model.address == "0xabc")
        #expect(service.enabledAssetIds == [ethereum.id.identifier])
    }

    @Test
    func aSlowerNetworkSwapDoesNotReplaceTheOneChosenAfterIt() async {
        let service = GemReceiveServiceMock()
        service.assetsById = [ethereum.id.identifier: ethereum.toGem(), solana.id.identifier: solana.toGem()]
        let model = ReceiveSceneViewModel.mock(service: service)

        model.onFinishNetworkSelection([ethereum.id])
        model.onFinishNetworkSelection([solana.id])
        await model.selectNetworkTask?.value
        await model.onChangeAsset()

        #expect(model.asset.chain == .solana)
        #expect(model.address == "So1ana")
        #expect(service.enabledAssetIds == [solana.id.identifier], "the network the user left is not enabled behind their back")
    }

    @Test
    func aFailedNetworkSwapShowsTheError() async {
        let service = GemReceiveServiceMock()
        service.assetResult = .failure(AnyError("asset is gone"))
        let model = ReceiveSceneViewModel.mock(service: service)

        model.onFinishNetworkSelection([ethereum.id])
        await model.selectNetworkTask?.value

        #expect(model.isPresentingAlertMessage?.message == "asset is gone")
        #expect(model.asset.chain == .bitcoin)
    }

    @Test
    func theSheetAndTheToastReadTheSamePresentation() {
        let model = ReceiveSceneViewModel.mock()

        model.onShareSheet()
        #expect(model.isPresentingSheet == .share)
        #expect(model.isPresentingCopyToast == false)

        model.onCopyAddress()
        #expect(model.isPresentingSheet == nil)
        #expect(model.isPresentingCopyToast)

        model.isPresentingCopyToast = false
        #expect(model.presentation == nil)
    }

    @Test
    func sharingSendsTheAddressWithTheCodeWhenThereIsOne() {
        let model = ReceiveSceneViewModel.mock()

        #expect(model.activityItems(qrImage: nil).count == 1)
        #expect(model.activityItems(qrImage: UIImage()).count == 2)
    }
}
