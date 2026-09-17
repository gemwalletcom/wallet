import Components
import Formatters
import GemstonePrimitives
import InfoSheet
import Localization
import GemstoneServices
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct PerpetualScene: View {
    @Environment(\.scenePhase) private var scenePhase

    @Bindable var model: PerpetualSceneViewModel

    public init(model: PerpetualSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        @Bindable var chart = model.chart

        List {
            Section {} header: {
                VStack {
                    VStack {
                        switch chart.state {
                        case .noData:
                            StateEmptyView(title: chart.emptyTitle, image: chart.emptyImage)
                        case .loading: LoadingView()
                        case let .data(data):
                            CandlestickChartView(
                                model: CandlestickChartViewModel(
                                    candles: data,
                                    period: chart.currentPeriod,
                                    position: model.positions.first?.position,
                                ),
                            )
                        case let .error(error):
                            StateEmptyView(
                                title: error.networkOrNoDataDescription,
                                image: Images.ErrorContent.error,
                            )
                        }
                    }
                    .frame(height: Sizing.chart.height)

                    PeriodSelectorView(selectedPeriod: $chart.currentPeriod)
                        .padding(.horizontal, Spacing.medium)
                }
            }
            .fullWidthSection()

            ForEach(model.sections, id: \.self) { section in
                switch section {
                case .position:
                    ForEach(model.positionViewModels) { position in
                        Section {
                            positionContent(position)
                        } header: {
                            Text(section.title)
                        }
                    }
                case .info:
                    buttonsSection
                    Section(header: Text(section.title)) {
                        ForEach(model.infoRows, id: \.self) { row in
                            ListItemView(
                                field: model.perpetualViewModel.infoField(for: row),
                                infoAction: model.infoAction(for: row),
                            )
                        }
                    }
                }
            }

            if !model.transactionSections.isEmpty {
                TransactionsList(sections: model.transactionSections, currency: model.currency)
                .listRowInsets(.assetListRowInsets)
            }
        }
        .navigationTitle(model.navigationTitle)
        .navigationBarTitleDisplayMode(.inline)
        .alertSheet($model.isPresentingAlertMessage)
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(type: $0)
        }
        .alert(
            model.modifyTitle,
            presenting: $model.isPresentingModifyAlert,
            sensoryFeedback: .warning,
            actions: { _ in
                ForEach(model.modifyButtonModels) { button in
                    Button(button.title, role: button.isDestructive ? .destructive : nil) {
                        model.onSelect(button)
                    }
                }
                Button(Localized.Common.cancel, role: .cancel) {}
            },
        )
        .refreshable {
            await model.load()
        }
        .onAppear {
            Task { await model.onAppear() }
        }
        .onDisappear {
            Task { await model.onDisappear() }
        }
        .onChange(of: scenePhase, model.onScenePhaseChange)
        .onChange(of: chart.currentPeriod, model.onPeriodChange)
    }

    @ViewBuilder
    private var buttonsSection: some View {
        Section {
            HStack(spacing: Spacing.medium) {
                ForEach(model.buttonModels) { button in
                    Button(button.title) { model.onSelect(button) }
                        .frame(maxWidth: .infinity)
                        .buttonStyle(style(for: button))
                }
            }
        }
    }

    private func style(for button: PerpetualButtonViewModel) -> ColorButtonStyle {
        switch button.style {
        case .green: .green()
        case .red: .red()
        case .blue: .blue()
        }
    }

    @ViewBuilder
    private func positionContent(_ position: PerpetualPositionViewModel) -> some View {
        ListAssetItemView(model: PerpetualPositionItemViewModel(model: position))

        ForEach(model.positionRows(position), id: \.self) { row in
            switch row {
            case .pnl:
                ListItemView(field: position.detailField(for: row))
                    .numericTransition(for: position.pnlWithPercentText)
            case .autoclose:
                NavigationCustomLink(
                    with: ListItemView(model: model.autocloseListItem(position, row: row)),
                    action: model.onSelectAutoclose,
                )
            case .size, .entryPrice, .liquidationPrice, .margin, .fundingPayments:
                ListItemView(
                    field: position.detailField(for: row),
                    infoAction: model.infoAction(for: row),
                )
            }
        }
    }
}
