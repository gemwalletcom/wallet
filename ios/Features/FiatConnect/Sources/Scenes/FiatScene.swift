import Components
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
                text: $model.amount,
                error: model.amountError,
                config: model.currencyInputConfig,
            )
            .padding(.top, .medium)
            .listGroupRowStyle()
            amountSelectorSection
            providerSection(model.providerModel(viewState))
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
                        ForEach(model.suggestedAmounts, id: \.amount) { suggestion in
                            Button(suggestion.text) {
                                model.onSelect(amount: Int(suggestion.amount))
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

    private func providerSection(_ provider: FiatProviderViewModel) -> some View {
        Section {
            switch provider.quotesState {
            case .noData:
                StateEmptyView(title: provider.emptyTitle)
            case .loading:
                ListItemLoadingView()
                    .id(UUID())
            case .data:
                if let quote = provider.selectedQuote {
                    let view = ListItemImageView(
                        title: model.providerTitle,
                        subtitle: quote.providerName,
                        assetImage: model.providerAssetImage(quote.provider),
                    )
                    if provider.allowSelectProvider {
                        NavigationCustomLink(
                            with: view,
                            action: model.onSelectFiatProviders,
                        )
                    } else {
                        view
                    }
                    ListItemView(model: model.rateListItem)
                }
            case let .error(error):
                ListItemErrorView(errorTitle: model.errorTitle, error: error)
            }
        }
    }
}
