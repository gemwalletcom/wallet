// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemChainSettingsSection
import PrimitivesComponents
import Style
import SwiftUI

public struct ChainSettingsScene: View {
    @State private var model: ChainSettingsSceneViewModel

    public init(model: ChainSettingsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            ForEach(model.sections, id: \.title) { section in
                Section(section.title) {
                    content(for: section)
                }
            }
        }
        .refreshable {
            await model.load()
        }
        .alert(
            model.deleteConfirmationTitle(for: model.nodeDelete?.host ?? ""),
            presenting: $model.nodeDelete,
            sensoryFeedback: .warning,
            actions: { _ in
                Button(
                    model.deleteButtonTitle,
                    role: .destructive,
                    action: onDeleteNode,
                )
            },
        )
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button(action: model.onPresentImportNode) {
                    Images.System.plus
                        .font(.body.weight(.semibold))
                }
            }
        }
        .sheet(isPresented: $model.isPresentingImportNode) {
            NavigationStack {
                AddNodeScene(model: model.addNodeModel(), onDismiss: model.onDismissImportNode)
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
        .navigationTitle(model.title)
        .listSectionSpacing(.compact)
        .taskOnce {
            Task { await model.load() }
        }
    }

    @ViewBuilder
    private func content(for section: GemChainSettingsSection) -> some View {
        switch section {
        case let .nodes(rows):
            ForEach(rows, id: \.node.url) { row in
                SelectionView(value: row.node.url, selection: row.node.isSelected ? row.node.url : nil, action: model.onSelectNode) {
                    ListItemView(model: row.listItem)
                }
                .contextMenu(
                    .copy(value: row.node.url),
                )
                .if(row.canDelete) {
                    $0.swipeActions(edge: .trailing) {
                        Button(model.deleteButtonTitle, role: .destructive) {
                            model.onSelectNodeForDeletion(row.node)
                        }
                        .tint(Colors.red)
                    }
                }
            }
        case let .explorers(rows):
            ForEach(rows, id: \.name) { explorer in
                ListItemSelectionView(
                    title: explorer.name,
                    titleExtra: .none,
                    titleTag: .none,
                    titleTagType: .none,
                    subtitle: .none,
                    subtitleExtra: .none,
                    value: explorer.name,
                    selection: explorer.isSelected ? explorer.name : .none,
                    action: model.onSelectExplorer(name:),
                )
            }
        }
    }
}

// MARK: - Actions

extension ChainSettingsScene {
    private func onDeleteNode() {
        Task { await model.onDeleteNode() }
    }
}
