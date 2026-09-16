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
struct ReceiveViewModelTests {
    private let bitcoin = Primitives.Asset.mock(id: .mock(.bitcoin))
    private let ethereum = Primitives.Asset.mock(id: .mock(.ethereum))

    private func settle(until condition: () -> Bool = { false }) async {
        for _ in 0 ..< 200 {
            await Task.yield()
            if condition() {
                return
            }
            try? await Task.sleep(for: .milliseconds(5))
        }
    }

    @Test
    func theNetworksComeFromCore() {
        let service = GemReceiveServiceMock()
        service.networkAssetIdsValue = [bitcoin.id.identifier, ethereum.id.identifier]
        let model = ReceiveViewModel.mock(service: service)

        #expect(model.networkAssetIds == [bitcoin.id, ethereum.id])
        #expect(model.showNetworkSelector)
    }

    @Test
    func oneNetworkHidesTheSelector() {
        let model = ReceiveViewModel.mock()

        #expect(model.showNetworkSelector == false)
    }

    @Test
    func theWarningsComeFromCore() {
        let service = GemReceiveServiceMock()
        service.warningsValue = [.noMemoRequired, .noDestinationTagRequired]
        let model = ReceiveViewModel.mock(service: service)

        #expect(model.warningMessage.contains(Localized.Wallet.Receive.noMemoRequired))
        #expect(model.warningMessage.contains(Localized.Wallet.Receive.noDestinationTagRequired))
    }

    @Test
    func noWarningsMeanNoMessage() {
        let model = ReceiveViewModel.mock()

        #expect(model.warningMessage.isEmpty)
    }

    @Test
    func openingTheSceneEnablesTheAssetAndSyncsItsNetworks() async {
        let service = GemReceiveServiceMock()
        service.syncedNetworkAssetIdsResult = .success([bitcoin.id.identifier, ethereum.id.identifier])
        let model = ReceiveViewModel.mock(service: service)
        #expect(model.showNetworkSelector == false)

        model.onTaskOnce()
        await settle(until: { model.showNetworkSelector })

        #expect(service.enabledAssetIds == [bitcoin.id.identifier])
        #expect(service.syncedAssetIds == [bitcoin.id.identifier])
        #expect(model.networkAssetIds == [bitcoin.id, ethereum.id])
    }

    @Test
    func aFailedNetworkSyncKeepsTheStoredNetworks() async {
        let service = GemReceiveServiceMock()
        service.networkAssetIdsValue = [bitcoin.id.identifier, ethereum.id.identifier]
        service.syncedNetworkAssetIdsResult = .failure(AnyError("offline"))
        let model = ReceiveViewModel.mock(service: service)

        model.onTaskOnce()
        await settle(until: { !service.syncedAssetIds.isEmpty })

        #expect(model.networkAssetIds == [bitcoin.id, ethereum.id])
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aFailedEnableLeavesNoAlert() async {
        let service = GemReceiveServiceMock()
        service.enableAssetError = AnyError("offline")
        let model = ReceiveViewModel.mock(service: service)

        model.onTaskOnce()
        await settle()

        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func pickingTheSameNetworkChangesNothing() async {
        let service = GemReceiveServiceMock()
        let model = ReceiveViewModel.mock(service: service)
        let address = model.addressShort

        model.onSelectNetwork()
        #expect(model.presentation == .networkSelector)

        model.onFinishNetworkSelection([ReceiveNetworkItem(assetId: bitcoin.id)])
        await settle()

        #expect(model.presentation == nil)
        #expect(model.addressShort == address)
        #expect(service.requestedAssetIds.isEmpty)
    }

    @Test
    func pickingAnotherNetworkSwapsTheAssetAndItsAddress() async {
        let service = GemReceiveServiceMock()
        service.assetResult = .success(ethereum.toGem())
        let model = ReceiveViewModel.mock(service: service)

        model.onFinishNetworkSelection([ReceiveNetworkItem(assetId: ethereum.id)])
        await settle(until: { !service.enabledAssetIds.isEmpty })

        #expect(service.requestedAssetIds == [ethereum.id.identifier])
        #expect(model.assetModel.asset.chain == .ethereum)
        #expect(model.address == "0xabc")
        #expect(service.enabledAssetIds == [ethereum.id.identifier])
    }

    @Test
    func aFailedNetworkSwapShowsTheError() async {
        let service = GemReceiveServiceMock()
        service.assetResult = .failure(AnyError("asset is gone"))
        let model = ReceiveViewModel.mock(service: service)

        model.onFinishNetworkSelection([ReceiveNetworkItem(assetId: ethereum.id)])
        await settle(until: { model.isPresentingAlertMessage != nil })

        #expect(model.isPresentingAlertMessage?.message == "asset is gone")
        #expect(model.assetModel.asset.chain == .bitcoin)
    }

    @Test
    func theSheetAndTheToastReadTheSamePresentation() {
        let model = ReceiveViewModel.mock()

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
        let model = ReceiveViewModel.mock()

        #expect(model.activityItems(qrImage: nil).count == 1)
        #expect(model.activityItems(qrImage: UIImage()).count == 2)
    }
}
