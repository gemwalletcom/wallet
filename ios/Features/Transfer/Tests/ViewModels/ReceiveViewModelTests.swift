// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Gemstone
import GemstonePrimitives
import Localization
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import SwiftUI
import Testing
@testable import Transfer

@MainActor
struct ReceiveViewModelTests {
    private let bitcoin = Primitives.Asset.mock(id: .mock(.bitcoin))
    private let ethereum = Primitives.Asset.mock(id: .mock(.ethereum))

    private func model(
        service: GemReceiveServiceMock,
        asset: Primitives.Asset? = nil,
        address: String = "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
    ) -> ReceiveViewModel {
        let asset = asset ?? bitcoin
        return ReceiveViewModel(
            assetAddress: AssetAddress(asset: asset, address: address),
            wallet: .mock(accounts: [.mock(chain: asset.chain, address: address), .mock(chain: .ethereum, address: "0xabc")]),
            service: service,
        )
    }

    private func settle(until condition: () -> Bool = { false }) async {
        for _ in 0 ..< 200 {
            await Task.yield()
            if condition() { return }
            try? await Task.sleep(for: .milliseconds(5))
        }
    }

    @Test
    func theNetworksComeFromCore() {
        let service = GemReceiveServiceMock()
        service.networkAssetIdsValue = [bitcoin.id.identifier, ethereum.id.identifier]
        let model = model(service: service)

        #expect(model.networkAssetIds == [bitcoin.id, ethereum.id])
        #expect(model.showNetworkSelector)
    }

    @Test
    func oneNetworkHidesTheSelector() {
        let model = model(service: GemReceiveServiceMock())

        #expect(model.showNetworkSelector == false)
    }

    @Test
    func theWarningsComeFromCore() {
        let service = GemReceiveServiceMock()
        service.warningsValue = [.noMemoRequired, .noDestinationTagRequired]
        let model = model(service: service)

        #expect(model.warningMessage.contains(Localized.Wallet.Receive.noMemoRequired))
        #expect(model.warningMessage.contains(Localized.Wallet.Receive.noDestinationTagRequired))
    }

    @Test
    func noWarningsMeanNoMessage() {
        let model = model(service: GemReceiveServiceMock())

        #expect(model.warningMessage.isEmpty)
    }

    @Test
    func openingTheSceneEnablesTheAssetAndPrefetchesTheOthers() async {
        let service = GemReceiveServiceMock()
        service.networkAssetIdsValue = [bitcoin.id.identifier, ethereum.id.identifier]
        let model = model(service: service)

        model.onTaskOnce()
        await settle(until: { !service.enabledAssetIds.isEmpty && !service.syncedAssetIds.isEmpty })

        #expect(service.enabledAssetIds == [bitcoin.id.identifier])
        #expect(service.syncedAssetIds == [[ethereum.id.identifier]])
    }

    @Test
    func aFailedEnableLeavesNoAlert() async {
        let service = GemReceiveServiceMock()
        service.enableAssetError = AnyError("offline")
        let model = model(service: service)

        model.onTaskOnce()
        await settle()

        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func pickingTheSameNetworkChangesNothing() async {
        let service = GemReceiveServiceMock()
        let model = model(service: service)
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
        let model = model(service: service)

        model.onFinishNetworkSelection([ReceiveNetworkItem(assetId: ethereum.id)])
        await settle(until: { !service.enabledAssetIds.isEmpty })

        #expect(service.requestedAssetIds == [ethereum.id.identifier])
        #expect(model.assetModel.asset.chain == .ethereum)
        #expect(service.enabledAssetIds == [ethereum.id.identifier])
    }

    @Test
    func aFailedNetworkSwapShowsTheError() async {
        let service = GemReceiveServiceMock()
        service.assetResult = .failure(AnyError("asset is gone"))
        let model = model(service: service)

        model.onFinishNetworkSelection([ReceiveNetworkItem(assetId: ethereum.id)])
        await settle(until: { model.isPresentingAlertMessage != nil })

        #expect(model.isPresentingAlertMessage?.message == "asset is gone")
        #expect(model.assetModel.asset.chain == .bitcoin)
    }

    @Test
    func theSheetAndTheToastReadTheSamePresentation() {
        let model = model(service: GemReceiveServiceMock())

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
        let model = model(service: GemReceiveServiceMock())

        #expect(model.activityItems(qrImage: nil).count == 1)
        #expect(model.activityItems(qrImage: UIImage()).count == 2)
    }
}
