import GemstonePrimitivesTestKit
import GemstoneServicesTestKit
import Localization
@testable import PriceAlerts
import PriceAlertsTestKit
import Primitives
import PrimitivesTestKit
@testable import Store
import Testing

@MainActor
struct SetPriceAlertViewModelTests {
    @Test
    func theSuggestionsComeFromCoreAndAreFormattedForTheCurrency() {
        let viewModel = SetPriceAlertViewModel.mock()
        #expect(viewModel.suggestions(viewModel.viewState).isEmpty)

        viewModel.assetQuery.value = .mock(price: .mock(price: 2000))
        let prices = viewModel.suggestions(viewModel.viewState)
        viewModel.type = .percentage
        let percentages = viewModel.suggestions(viewModel.viewState)

        #expect(percentages.isNotEmpty)
        #expect(prices.isNotEmpty)
        #expect(prices.allSatisfy { $0.title.contains("$") })
        #expect(percentages.allSatisfy { $0.title.contains("%") })
    }

    @Test
    func pickingASuggestionFillsTheAmount() {
        let viewModel = SetPriceAlertViewModel.mock()

        viewModel.onSelectSuggestion(PriceSuggestion(title: "$67,000", value: 67000))

        #expect(viewModel.amount == "67000")
    }

    @Test
    func theConfirmButtonFollowsWhetherTheAlertCanBeSaved() {
        let viewModel = SetPriceAlertViewModel.mock()

        #expect(viewModel.confirmButtonState(viewModel.viewState) == .disabled)

        viewModel.assetQuery.value = .mock(price: .mock(price: 2000))
        viewModel.amount = "2500"

        #expect(viewModel.confirmButtonState(viewModel.viewState) == .normal)
    }

    @Test
    func savingEnablesTheAlertAndReportsIt() async {
        let service = GemPriceAlertServiceMock()
        let messages = MessageRecorder()
        let viewModel = SetPriceAlertViewModel.mock(service: service, onComplete: { messages.record($0) })
        viewModel.assetQuery.value = .mock(price: .mock(price: 2000))
        viewModel.amount = "2500"

        await viewModel.setPriceAlert()

        #expect(service.isEnabled())
        #expect(messages.messages.count == 1)
        #expect(messages.messages.first?.hasPrefix(Localized.PriceAlerts.addedPriceOver("")) == true, "one whole sentence, not a lowercased title")
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
    func eachAlertTypeKeepsItsOwnAmount() {
        let viewModel = SetPriceAlertViewModel.mock()
        viewModel.assetQuery.value = .mock(price: .mock(price: 2000))

        viewModel.amount = "2500"
        viewModel.type = .percentage
        #expect(viewModel.amount.isEmpty)
        #expect(viewModel.confirmButtonState(viewModel.viewState) == .disabled, "the percentage alert has no amount yet")

        viewModel.amount = "10"
        viewModel.type = .price
        #expect(viewModel.amount == "2500")
        #expect(viewModel.confirmButtonState(viewModel.viewState) == .normal)
    }
}

private final class MessageRecorder: @unchecked Sendable {
    private(set) var messages: [String] = []

    func record(_ message: String) {
        messages.append(message)
    }
}
