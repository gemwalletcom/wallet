import Components
import struct Gemstone.GemFiatViewState
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct FiatScene: View {
    @State private var model: FiatSceneViewModel

    public init(model: FiatSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let viewState = model.viewState
        return List {
            CurrencyInputValidationView(
                model: $model.inputValidationModel,
                config: model.currencyInputConfig,
            )
            .padding(.top, .medium)
            .listGroupRowStyle()
            amountSelectorSection
            providerSection(viewState)
        }
        .safeAreaButton {
            StateButton(
                text: model.actionButtonTitle(viewState),
                type: .primary(model.actionButtonState(viewState)),
                action: model.onSelectContinue,
            )
        }
        .contentMargins([.top], .zero, for: .scrollContent)
        .frame(maxWidth: .infinity)
        .onChange(of: model.type, model.onChangeType)
        .onChange(of: model.inputValidationModel.text, model.onChangeAmountText)
        .debouncedTask(id: model.loadTrigger, interval: model.quoteDebounce) {
            await model.load()
        }
        .onTimer(every: model.quoteRefreshInterval, id: model.loadTrigger) {
            await model.load()
        }
        .alertSheet($model.isPresentingAlertMessage)
    }
}

// MARK: - UI Components

extension FiatScene {
    private var amountSelectorSection: some View {
        Section {
            AssetBalanceView(
                image: model.assetImage,
                title: model.assetTitle,
                balance: model.assetBalance,
                secondary: {
                    HStack(spacing: .space10) {
                        ForEach(model.suggestedAmounts, id: \.self) { amount in
                            Button(model.buttonTitle(amount: amount)) {
                                model.onSelect(amount: amount)
                            }
                            .font(.subheadline.weight(.semibold))
                            .buttonStyle(.amount())
                        }

                        Button(model.typeAmountButtonTitle) {
                            model.onSelectRandomAmount()
                        }
                        .font(.subheadline.weight(.semibold))
                        .buttonStyle(.listEmpty())
                        .overlay {
                            RandomOverlayView()
                        }
                    }
                    .fixedSize()
                },
            )
        }
    }

    private func providerSection(_ viewState: GemFiatViewState) -> some View {
        Section {
            switch model.quotesState(viewState) {
            case .noData:
                StateEmptyView(title: model.emptyTitle(viewState))
            case .loading:
                ListItemLoadingView()
                    .id(UUID())
            case .data:
                if let quote = model.selectedQuote(viewState) {
                    let view = ListItemImageView(
                        title: model.providerTitle,
                        subtitle: quote.providerName,
                        assetImage: model.providerAssetImage(quote.provider),
                    )
                    if model.allowSelectProvider(viewState) {
                        NavigationCustomLink(
                            with: view,
                            action: model.onSelectFiatProviders,
                        )
                    } else {
                        view
                    }
                    ListItemView(title: model.rateTitle, subtitle: model.rateValue)
                }
            case let .error(error):
                ListItemErrorView(errorTitle: model.errorTitle, error: error)
            }
        }
    }
}
