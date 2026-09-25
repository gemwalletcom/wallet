// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemCollectibleDetails
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct CollectibleScene: View {
    @State private var model: CollectibleViewModel

    public init(model: CollectibleViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let details = model.details
        return List {
            headerSectionView(details)
            ForEach(details.sections, id: \.self) { group in
                switch group.section {
                case let .status(status):
                    Section {
                        AssetStatusView(status: status.toPrimitives(), action: model.onSelectStatus)
                    }
                case let .info(rows):
                    Section {
                        ForEach(rows, id: \.self) { row in
                            GemListRowView(row: row, onSelectAddress: model.onSelectContract)
                        }
                    }
                case let .attributes(attributes):
                    Section(group.title.text ?? .empty) {
                        ForEach(attributes, id: \.self) {
                            ListItemView(model: model.attributeListItem($0))
                        }
                    }
                case let .links(links):
                    Section(group.title.text ?? .empty) {
                        SocialLinksView(links: links)
                    }
                }
            }
        }
        .environment(\.defaultMinListHeaderHeight, 0)
        .listSectionSpacing(.compact)
        .contentMargins([.top], .small, for: .scrollContent)
        .navigationTitle(model.title)
        .toolbar {
            ToolbarItem(placement: .principal) {
                HStack(spacing: .tiny) {
                    Text(model.title)
                        .font(.headline)
                    if details.isVerified {
                        VerifiedBadgeView(font: .subheadline)
                    }
                }
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
        .toast(message: $model.isPresentingToast)
        .sheet(isPresented: $model.isPresentingReportSheet) {
            ReportNavigationStack(model: model.reportModel())
        }
        .sheet(item: $model.isPresentingInfoSheet) {
            InfoSheetScene(sheet: $0)
        }
        .bindQuery(model.query)
    }
}

// MARK: - UI

extension CollectibleScene {
    private func headerSectionView(_ details: GemCollectibleDetails) -> some View {
        Section {
            NftImageView(
                assetImage: model.assetImage,
                isImageLoaded: $model.isImageLoaded,
            )
            .aspectRatio(1, contentMode: .fill)
        } header: {
            Spacer()
        } footer: {
            HeaderButtonsView(buttons: model.headerButtons(details), action: model.onSelectHeaderButton(type:))
                .padding(.top, .medium)
                .padding(.bottom, .small)
        }
        .frame(maxWidth: .infinity)
        .textCase(nil)
        .listRowSeparator(.hidden)
        .listRowInsets(EdgeInsets())
        .contextMenu(model.imageContextMenuItems(details))
    }
}
