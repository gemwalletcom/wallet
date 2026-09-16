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

    @Test
    func theSuggestionsComeFromCoreAndAreFormattedForTheCurrency() {
        let viewModel = SetPriceAlertViewModel.mock()
        let price = Price.mock(price: 2000)

        let percentages = viewModel.percentageSuggestions(for: price)
        let prices = viewModel.priceSuggestions(for: price)

        #expect(percentages.isNotEmpty)
        #expect(prices.isNotEmpty)
        #expect(prices.allSatisfy { $0.title.contains("$") })
        #expect(viewModel.priceSuggestions(for: nil).isEmpty)
    }

    @Test
    func pickingASuggestionFillsTheAmount() {
        let viewModel = SetPriceAlertViewModel.mock()

        viewModel.onSelectSuggestion(PriceSuggestion(title: "$67,000", value: 67000))

        #expect(viewModel.state.amount == "67000")
    }

    @Test
    func theConfirmButtonFollowsWhetherTheAlertCanBeSaved() {
        let viewModel = SetPriceAlertViewModel.mock()

        #expect(viewModel.confirmButtonState == .disabled)

        viewModel.assetQuery.value = .mock(price: .mock(price: 2000))
        viewModel.state.amount = "2500"

        #expect(viewModel.confirmButtonState == .normal)
    }

    @Test
    func savingEnablesTheAlertAndReportsIt() async {
        let service = GemPriceAlertServiceMock()
        let messages = MessageRecorder()
        let viewModel = SetPriceAlertViewModel.mock(service: service, onComplete: { messages.record($0) })
        viewModel.assetQuery.value = .mock(price: .mock(price: 2000))
        viewModel.state.amount = "2500"

        await viewModel.setPriceAlert()

        #expect(service.isEnabled())
        #expect(messages.messages.count == 1)
        #expect(viewModel.isPresentingAlertMessage == nil)
    }

    @Test
    func savingWithNoAmountDoesNothing() async {
        let service = GemPriceAlertServiceMock()
        let viewModel = SetPriceAlertViewModel.mock(service: service)

        await viewModel.setPriceAlert()

        #expect(service.isEnabled() == false)
    }

    @Test
    func changingTheAlertTypeKeepsTheChosenType() {
        let viewModel = SetPriceAlertViewModel.mock()

        viewModel.onChangeAlertType(.price, type: .percentage)

        #expect(viewModel.state.type == .percentage)
    }
}

private final class MessageRecorder: @unchecked Sendable {
    private(set) var messages: [String] = []

    func record(_ message: String) {
        messages.append(message)
    }
}

private extension SetPriceAlertViewModel {
    static func mock(
        service: any GemPriceAlertServiceProtocol = GemPriceAlertServiceMock(),
        onComplete: @escaping (String) -> Void = { _ in },
    ) -> SetPriceAlertViewModel {
        SetPriceAlertViewModel(
            walletId: WalletId.mock(),
            asset: .mock(),
            service: service,
            onComplete: onComplete,
        )
    }
}
