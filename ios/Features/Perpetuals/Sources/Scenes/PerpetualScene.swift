import Components
import Formatters
import struct Gemstone.GemPerpetualButtonRow
import struct Gemstone.GemPerpetualPositionDetail
import struct Gemstone.GemPerpetualPositionRow
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
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
        let details = model.details

        return List {
            Section {} header: {
                VStack {
                    VStack {
                        switch chart.state(position: details.position) {
                        case .noData:
                            StateEmptyView(title: chart.emptyTitle, image: chart.emptyImage)
                        case .loading: LoadingView()
                        case let .data(data):
                            CandlestickChartView(chart: data)
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

            ForEach(details.sections, id: \.self) { section in
                switch section {
                case let .position(rows):
                    if let positionRow = details.positionRow {
                        Section {
                            positionContent(positionRow, rows: rows)
                        } header: {
                            Text(section.title)
                        }
                    }
                case let .info(buttons, rows):
                    buttonsSection(buttons)
                    Section(header: Text(section.title)) {
                        ForEach(rows, id: \.self) { row in
                            GemListRowView(row: row, onInfo: model.onInfo)
                        }
                    }
                }
            }

            if !model.transactionSections.isEmpty {
                TransactionsList(sections: model.transactionSections)
                    .listRowInsets(.assetListRowInsets)
            }
        }
        .navigationTitle(details.title)
        .navigationBarTitleDisplayMode(.inline)
        .alertSheet($model.isPresentingAlertMessage)
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(sheet: $0)
        }
        .alert(
            model.modifyTitle,
            presenting: $model.isPresentingModifyAlert,
            sensoryFeedback: .warning,
            actions: { _ in
                ForEach(details.modifyButtons, id: \.button) { row in
                    Button(row.button.title, role: row.tone == .negative ? .destructive : nil) {
                        model.onSelectButton(row.button)
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

    private func buttonsSection(_ buttons: [GemPerpetualButtonRow]) -> some View {
        Section {
            HStack(spacing: Spacing.medium) {
                ForEach(buttons, id: \.button) { row in
                    Button(row.button.title) { model.onSelectButton(row.button) }
                        .frame(maxWidth: .infinity)
                        .buttonStyle(row.tone.buttonStyle)
                }
            }
        }
    }

    @ViewBuilder
    private func positionContent(_ row: GemPerpetualPositionRow, rows: [GemPerpetualPositionDetail]) -> some View {
        ListAssetItemView(row: row.row)

        ForEach(rows, id: \.kind) { detail in
            switch detail.kind {
            case .pnl:
                GemListRowView(row: detail.row)
                    .numericTransition(for: detail.row)
            case .autoclose:
                NavigationCustomLink(
                    with: GemListRowView(row: detail.row, onInfo: model.onInfo),
                    action: model.onSelectAutoclose,
                )
            case .size, .entryPrice, .liquidationPrice, .margin, .fundingPayments:
                GemListRowView(row: detail.row, onInfo: model.onInfo)
            }
        }
    }
}
