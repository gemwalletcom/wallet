// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Gemstone
import GemstonePrimitivesTestKit
import GemstonePrimitives
import InfoSheet
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import StoreTestKit
import Testing

@MainActor
struct PerpetualSceneViewModelTests {
    @Test
    func theTitleFallsBackToTheAssetSymbol() {
        let model = PerpetualSceneViewModel.mock(asset: .mock(symbol: "HYPE"))

        #expect(model.navigationTitle == "HYPE")
    }

    @Test
    func theSceneReadsItsRowsFromCore() {
        let service = GemPerpetualDetailsServiceMock()
        service.sectionsValue = [.position]
        service.buttonsValue = [.long, .short]
        service.modifyButtonsValue = [.increase, .reduce]
        service.infoRowsValue = [.loading]
        let model = PerpetualSceneViewModel.mock(service: service)

        #expect(model.sections == [.position])
        #expect(model.buttons == [.long, .short])
        #expect(model.modifyButtons == [.increase, .reduce])
        #expect(model.infoRows == [.loading])
    }

    @Test
    func onlyTheRowsWithAnExplanationOfferOne() {
        let model = PerpetualSceneViewModel.mock()

        #expect(model.infoAction(for: GemPerpetualPositionDetailRow.pnl) == nil)
        #expect(model.infoAction(for: GemPerpetualPositionDetailRow.autoclose) != nil)
        #expect(model.infoAction(for: GemPerpetualPositionDetailRow.liquidationPrice) != nil)
        #expect(model.infoAction(for: GemPerpetualPositionDetailRow.fundingPayments) != nil)
    }

    @Test
    func openingALongPositionPassesTheDirectionToCore() {
        let service = GemPerpetualDetailsServiceMock()
        var actions: [GemPerpetualPositionAction] = []
        let model = PerpetualSceneViewModel.mock(service: service, onPerpetualPosition: { actions.append($0) })

        model.onSelectButton(.long)

        #expect(service.positionKinds == [.open(direction: PerpetualDirection.long.toGem())])
        #expect(actions.count == 1)
    }

    @Test
    func openingAShortPositionPassesTheDirectionToCore() {
        let service = GemPerpetualDetailsServiceMock()
        let model = PerpetualSceneViewModel.mock(service: service)

        model.onSelectButton(.short)

        #expect(service.positionKinds == [.open(direction: PerpetualDirection.short.toGem())])
    }

    @Test
    func modifyingOffersTheChoiceThenClosesItOnPick() {
        let service = GemPerpetualDetailsServiceMock()
        let model = PerpetualSceneViewModel.mock(service: service)

        model.onSelectButton(.modify)
        #expect(model.isPresentingModifyAlert == true)

        model.onSelectButton(.increase)
        #expect(model.isPresentingModifyAlert == false)
        #expect(service.positionKinds == [.increase])

        model.onSelectButton(.modify)
        model.onSelectButton(.reduce)
        #expect(model.isPresentingModifyAlert == false)
        #expect(service.positionKinds == [.increase, .reduce])
    }

    @Test
    func aFailedPositionActionShowsTheError() {
        let service = GemPerpetualDetailsServiceMock()
        service.positionActionResult = .failure(AnyError("no margin"))
        var actions: [GemPerpetualPositionAction] = []
        let model = PerpetualSceneViewModel.mock(service: service, onPerpetualPosition: { actions.append($0) })

        model.onSelectButton(.long)

        #expect(actions.isEmpty)
        #expect(model.isPresentingAlertMessage?.message == "no margin")
    }

    @Test
    func closingSendsTheTransferCoreBuilt() {
        let service = GemPerpetualDetailsServiceMock()
        var transfers: [GemTransferData] = []
        let model = PerpetualSceneViewModel.mock(service: service, onTransferData: { transfers.append($0) })

        model.onSelectButton(.close)

        #expect(transfers.count == 1)
        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func aFailedCloseShowsTheError() {
        let service = GemPerpetualDetailsServiceMock()
        service.closeTransferResult = .failure(AnyError("position is gone"))
        var transfers: [GemTransferData] = []
        let model = PerpetualSceneViewModel.mock(service: service, onTransferData: { transfers.append($0) })

        model.onSelectButton(.close)

        #expect(transfers.isEmpty)
        #expect(model.isPresentingAlertMessage?.message == "position is gone")
    }

    @Test
    func theInfoSheetsMatchTheRowThatOpenedThem() {
        let model = PerpetualSceneViewModel.mock()

        model.onInfo(.fundingApr)
        #expect(model.isPresentingInfoSheet == .fundingApr)

        model.onSelectFundingPaymentsInfo()
        #expect(model.isPresentingInfoSheet == .fundingPayments)

        model.onSelectLiquidationPriceInfo()
        #expect(model.isPresentingInfoSheet == .liquidationPrice)

        model.onInfo(.openInterest)
        #expect(model.isPresentingInfoSheet == .openInterest)

        model.onSelectAutocloseInfo()
        #expect(model.isPresentingInfoSheet == .autoclose)
    }

    @Test
    func loadingSyncsPositionsAndTransactionsForTheAsset() async {
        let service = GemPerpetualDetailsServiceMock()
        let asset = Primitives.Asset.mock()
        let model = PerpetualSceneViewModel.mock(service: service, asset: asset)

        await model.load()

        #expect(service.syncPositionsCount == 1)
        #expect(service.syncedTransactionAssetIds == [asset.id.identifier])
    }

    @Test
    func aFailedSyncIsSwallowedAndLeavesNoAlert() async {
        let service = GemPerpetualDetailsServiceMock()
        service.syncPositionsError = AnyError("offline")
        service.syncTransactionsError = AnyError("offline")
        let model = PerpetualSceneViewModel.mock(service: service)

        await model.load()

        #expect(model.isPresentingAlertMessage == nil)
    }

    @Test
    func dismissingAutocloseClearsThePresentedPosition() {
        let model = PerpetualSceneViewModel.mock()

        model.onAutocloseComplete()

        #expect(model.isPresentingAutoclose == nil)
    }
}
