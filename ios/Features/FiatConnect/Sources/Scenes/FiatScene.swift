import Components
import struct Gemstone.GemFiatViewState
import GemstonePrimitives
import InfoSheet
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

struct FiatScene: View {
    @State private var model: FiatSceneViewModel

    init(model: FiatSceneViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        let viewState = model.viewState
        return List {
            CurrencyInputValidationView(
                text: $model.amount,
                error: model.amountError(viewState),
                config: model.currencyInputConfig(viewState),
            )
            .padding(.top, .medium)
            .listGroupRowStyle()
            amountSelectorSection
            providerSection(viewState)
        }
        .safeAreaButton {
            StateButton(
                text: viewState.buttonAction.title,
                type: .primary(viewState.buttonState.state),
                action: { Task { await model.onSelectContinue() } },
            )
        }
        .contentMargins([.top], .zero, for: .scrollContent)
        .frame(maxWidth: .infinity)
        .onChange(of: model.type, model.onChangeType)
        .debouncedTask(id: model.loadTrigger, interval: GemConstants.fiatQuoteDebounce) {
            await model.load()
        }
        .onTimer(every: GemConstants.fiatQuoteRefreshInterval.timeInterval, id: model.loadTrigger) {
            await model.refreshQuotes()
        }
        .sheet(item: $model.isPresentingInfoSheet) { InfoSheetScene(model: $0) }
    }
}

// MARK: - UI Components

extension FiatScene {
    private var amountSelectorSection: some View {
        Section {
            ViewThatFits(in: .horizontal) {
                assetBalanceView(isCompact: false)
                assetBalanceView(isCompact: true)
            }
        }
    }

    private func assetBalanceView(isCompact: Bool) -> some View {
        let font: Font = isCompact ? .footnote.weight(.semibold) : .subheadline.weight(.semibold)
        let padding: CGFloat = isCompact ? .space6 : .small
        return AssetBalanceView(
            image: model.assetImage,
            title: model.assetTitle,
            balance: model.assetBalance,
            secondary: {
                HStack(spacing: isCompact ? .small : .space10) {
                    ForEach(model.suggestedAmounts, id: \.amount) { suggestion in
                        Button(suggestion.value.text()) {
                            model.onSelect(amount: Int(suggestion.amount))
                        }
                        .font(font)
                        .buttonStyle(.amount(paddingHorizontal: padding))
                    }

                    Button(model.typeAmountButtonTitle) {
                        model.onSelectRandomAmount()
                    }
                    .font(font)
                    .buttonStyle(.listEmpty(paddingHorizontal: padding))
                    .overlay {
                        RandomOverlayView()
                    }
                }
                .fixedSize()
            },
        )
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
                if let quote = viewState.selectedQuoteRow {
                    let view = ListItemImageView(
                        title: model.providerTitle,
                        subtitle: quote.providerName,
                        assetImage: model.providerAssetImage(quote.provider),
                    )
                    if viewState.canSelectProvider {
                        NavigationCustomLink(
                            with: view,
                            action: model.onSelectFiatProviders,
                        )
                    } else {
                        view
                    }
                    if let rateRow = viewState.rateRow {
                        GemListRowView(row: rateRow)
                    }
                }
            case let .error(error):
                ListItemErrorView(errorTitle: model.errorTitle, error: error)
            }
        }
    }
}
