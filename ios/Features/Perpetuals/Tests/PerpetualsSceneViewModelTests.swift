// Copyright (c). Gem Wallet. All rights reserved.

import Components
import func Gemstone.emptyState
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Localization
@testable import Perpetuals
import PerpetualsTestKit
import Primitives
import PrimitivesTestKit
import Testing

@MainActor
struct PerpetualsSceneViewModelTests {
    @Test
    func pullToRefreshUpdatesMarketsThatTheTimerWouldSkip() async {
        let perpetuals = GemPerpetualServiceMock()
        let model = PerpetualsSceneViewModel.mock(perpetualService: perpetuals)

        await model.load(source: .timer)
        #expect(perpetuals.syncMarketsCount == 1)

        await model.load(source: .timer)
        #expect(perpetuals.syncMarketsCount == 1)

        await model.load(source: .user)
        #expect(perpetuals.syncMarketsCount == 2)
    }

    @Test
    func aSearchThatMatchesNothingShowsTheSearchEmptyState() {
        let model = PerpetualsSceneViewModel.mock()

        #expect(model.marketView.sections == [.header])
        #expect(model.marketView.phase == .rows)

        model.isSearching = true

        #expect(model.marketView.sections.isEmpty)
        #expect(model.marketView.phase == .empty(state: emptyState(kind: .searchPerpetuals)))
    }

    @Test
    func openingTheSceneSyncsPositionsAndMarkets() async {
        let perpetuals = GemPerpetualServiceMock()
        let model = PerpetualsSceneViewModel.mock(perpetualService: perpetuals)

        await model.load()

        #expect(perpetuals.syncPositionsCount == 1)
        #expect(perpetuals.syncMarketsCount == 1)
    }

    @Test
    func depositOpensThePickerWhenThereIsAChoice() async {
        var selected: SelectAssetType?
        let model = PerpetualsSceneViewModel.mock(onSelectAssetType: { selected = $0 })

        await model.onSelectDeposit()

        #expect(selected == .deposit)
    }

    @Test
    func depositOpensTheAmountOfTheOneSource() async {
        let perpetuals = GemPerpetualServiceMock()
        perpetuals.depositTargetValue = .amount(asset: Asset.mock().toGem())
        var selected: AmountInput?
        let model = PerpetualsSceneViewModel.mock(onSelectAmount: { selected = $0 }, perpetualService: perpetuals)

        await model.onSelectDeposit()

        #expect(selected == AmountInput(type: .deposit, asset: .mock()))
    }

    @Test
    func unavailablePerpetualsBlocksDepositButKeepsWithdrawal() async {
        let service = GemPerpetualServiceMock()
        service.isAvailableValue = false
        service.depositTargetValue = .amount(asset: Asset.mock().toGem())
        var amounts: [AmountInput] = []
        let model = PerpetualsSceneViewModel.mock(onSelectAmount: { amounts.append($0) }, perpetualService: service)

        #expect(model.isPresentingInfoSheet == nil)
        await model.onSelectDeposit()
        #expect(amounts.isEmpty)
        #expect(model.isPresentingInfoSheet?.description == Localized.Info.regionUnavailableDescription)

        model.isPresentingInfoSheet = nil
        model.onSelectHeaderAction(.withdraw(asset: Asset.mock().toGem()))
        #expect(amounts.count == 1)
        #expect(model.isPresentingInfoSheet == nil)
        #expect(service.availabilityCheckCount == 1)

        service.isAvailableValue = true
        await model.onSelectDeposit()
        #expect(amounts.count == 2)
    }
}
