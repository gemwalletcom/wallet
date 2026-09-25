// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Gemstone
import GemstonePrimitives
import GemstonePrimitivesTestKit
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
    func theSceneReadsItsScreenFromCore() {
        let service = GemPerpetualDetailsServiceMock()
        service.detailsValue = .mock(
            title: "HYPE",
            sections: [.info(buttons: [GemPerpetualButtonRow(button: .long, tone: .positive), GemPerpetualButtonRow(button: .short, tone: .negative)], rows: [.loading])],
            modifyButtons: [GemPerpetualButtonRow(button: .increase, tone: .neutral), GemPerpetualButtonRow(button: .reduce, tone: .negative)],
        )
        let model = PerpetualSceneViewModel.mock(service: service)

        #expect(model.details == service.detailsValue)
        #expect(model.positionData(model.details) == nil)
    }

    @Test
    func buttonsAreDrawnInTheToneCoreGives() {
        let model = PerpetualSceneViewModel.mock(service: GemPerpetualDetailsServiceMock())
        let buttons = model.buttonModels([
            GemPerpetualButtonRow(button: .long, tone: .positive),
            GemPerpetualButtonRow(button: .increase, tone: .neutral),
            GemPerpetualButtonRow(button: .reduce, tone: .negative),
        ])

        #expect(buttons.map(\.style) == [.green, .blue, .red])
        #expect(buttons.map(\.isDestructive) == [false, false, true])
    }

    @Test
    func thePositionCoreNamesIsTheOneTheSceneShowsAndActsOn() {
        let service = GemPerpetualDetailsServiceMock()
        let position = Primitives.PerpetualPosition.mock()
        service.detailsValue = .mock(sections: [.position(rows: [])], position: position.toGem())
        let model = PerpetualSceneViewModel.mock(service: service)

        #expect(model.positionData(model.details)?.position == position)

        model.onSelectAutoclose()
        #expect(model.isPresentingAutoclose?.position == position)
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
        #expect(model.isPresentingInfoSheet?.title == .fundingApr)

        model.onInfo(.fundingPayments)
        #expect(model.isPresentingInfoSheet?.title == .fundingPayments)

        model.onInfo(.liquidationPrice)
        #expect(model.isPresentingInfoSheet?.title == .liquidationPrice)

        model.onInfo(.openInterest)
        #expect(model.isPresentingInfoSheet?.title == .openInterest)

        model.onInfo(.autoClose)
        #expect(model.isPresentingInfoSheet?.title == .autoClose)
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
    func comingBackToTheScreenSyncsTheSameTwoThingsAsOpeningIt() async {
        let service = GemPerpetualDetailsServiceMock()
        let asset = Primitives.Asset.mock()
        let model = PerpetualSceneViewModel.mock(service: service, asset: asset)

        model.onScenePhaseChange(.background, .active)
        for _ in 0 ..< 200 where service.syncPositionsCount == 0 {
            try? await Task.sleep(for: .milliseconds(20))
        }

        #expect(service.syncPositionsCount == 1, "a position closed while the app was away shows on return")
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
