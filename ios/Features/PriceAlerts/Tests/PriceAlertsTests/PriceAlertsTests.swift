import protocol Gemstone.GemPriceAlertServiceProtocol
import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
@testable import PriceAlerts
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
struct SetPriceAlertViewModelTests {
    @Test
    func alertDirectionFromEnteredAmountAndAssetPrice() {
        let viewModel = SetPriceAlertViewModel.mock()
        viewModel.assetQuery.value = .mock(price: .mock(price: 2119.15))

        viewModel.state.amount = "1233"
        #expect(viewModel.alertDirection == .down)
        #expect(viewModel.isEnabledConfirmButton)

        viewModel.state.amount = "3000,00"
        #expect(viewModel.alertDirection == .up)

        viewModel.state.amount = ""
        #expect(viewModel.alertDirection == nil)
        #expect(viewModel.isEnabledConfirmButton == false)
    }

    @Test
    func confirmDisabledWithoutAssetPrice() {
        let viewModel = SetPriceAlertViewModel.mock()
        viewModel.state.amount = "200"

        #expect(viewModel.alertDirection == nil)
        #expect(viewModel.isEnabledConfirmButton == false)
    }

    @Test
    func percentageAlertUsesSelectedDirection() {
        let viewModel = SetPriceAlertViewModel.mock()
        viewModel.state.type = .percentage
        viewModel.state.amount = "5"

        #expect(viewModel.alertDirection == .up)
        #expect(viewModel.isEnabledConfirmButton)

        viewModel.state.selectedDirection = .down
        #expect(viewModel.alertDirection == .down)
    }
}

private extension SetPriceAlertViewModel {
    static func mock() -> SetPriceAlertViewModel {
        SetPriceAlertViewModel(
            walletId: WalletId.mock(),
            asset: .mock(),
            priceAlertService: GemPriceAlertServiceMock(),
            preferencesService: GemPreferencesServiceMock(),
            onComplete: { _ in },
        )
    }
}
