// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemDurationPart
import enum Gemstone.GemListRow
import enum Gemstone.GemStakeDestination
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import GemstoneServices
import GemstoneServicesTestKit
import Localization
import Primitives
import PrimitivesTestKit
@testable import Stake
import StakeTestKit
@testable import Store
import Testing

@MainActor
struct StakeSceneViewModelTests {
    @Test
    func theInfoSectionShowsTheRowsCoreReturns() {
        let rows: [GemListRow] = [
            .amount(title: .stakeApr, amount: .mock(value: 12.5, tone: .positive), info: .stakeApr),
            .duration(title: .lockTime, parts: [GemDurationPart(value: 14, unit: .day)], info: .stakeLockTime, estimate: false),
        ]
        let model = StakeSceneViewModel.mock(chain: .tron, stakeService: GemStakeServiceMock(infoRows: rows))

        #expect(model.viewState.infoRows == rows)
    }

    @Test
    func theInfoSheetMatchesTheRowThatOpenedIt() {
        let model = StakeSceneViewModel.mock(chain: .tron)

        model.onInfo(.stakeApr)
        #expect(model.isPresentingInfoSheet?.id == "stakeApr")

        model.onInfo(.stakeLockTime)
        #expect(model.isPresentingInfoSheet?.id == "stakeLockTime")
    }

    @Test
    func aFailedRefreshWithNothingShownShowsTheError() async {
        let model = StakeSceneViewModel.mock(chain: .tron, stakeService: GemStakeServiceMock(refreshState: .error(error: .Gateway(msg: "offline"))))

        await model.load()

        guard case .error = model.delegationsViewState(model.viewState) else {
            Issue.record("expected the refresh error")
            return
        }
    }

    @Test
    func stakeStillRequiresValidators() {
        let tron = StakeSceneViewModel.mock(chain: .tron)
        tron.assetQuery.value = .mock(asset: Chain.tron.asset, balance: .mock(frozen: 1))

        let stake = tron.viewState.actions.first { $0.action == .stake }
        guard case .disabled = stake?.tap else {
            Issue.record("expected a disabled stake action")
            return
        }
    }

    @Test
    func claimRewardsRoutesToConfirm() {
        let transfer = GemTransferData.mock()
        let model = StakeSceneViewModel.mock(chain: .tron, stakeService: GemStakeServiceMock(claimRewardsDestination: .confirm(transfer: transfer)))

        #expect(claimDestination(model).map(model.route(destination:)) == .transfer(.confirm(transfer)))
    }

    @Test
    func claimRewardsAcrossValidatorsRoutesToAmount() {
        let model = StakeSceneViewModel.mock(chain: .tron)

        guard case .transfer(.amount) = claimDestination(model).map(model.route(destination:)) else {
            Issue.record("expected an amount route")
            return
        }
    }

    private func claimDestination(_ model: StakeSceneViewModel) -> GemStakeDestination? {
        guard case let .open(destination) = model.viewState.actions.first(where: { $0.action == .claimRewards })?.tap else { return nil }
        return destination
    }
}
