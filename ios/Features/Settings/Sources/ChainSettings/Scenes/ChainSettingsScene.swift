// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListItem
import struct Gemstone.GemNodeRow
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
            GemGroupSections(groups: model.sections) { item in
                content(for: item)
            }
        }
        .refreshable {
            await model.load()
        }
        .alert(
            model.nodeDelete?.deletePrompt.text ?? "",
            presenting: $model.nodeDelete,
            sensoryFeedback: .warning,
            actions: { row in
                Button(
                    model.deleteButtonTitle,
                    role: .destructive,
                    action: { onDeleteNode(row) },
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
    private func content(for item: GemListItem) -> some View {
        switch item {
        case let .node(row):
            SelectionView(value: row.node.url, selection: row.node.isSelected ? row.node.url : nil, action: model.onSelectNode) {
                ListItemView(model: row.listItem)
            }
            .contextMenu(
                .copy(value: row.node.url),
            )
            .if(row.canDelete) {
                $0.swipeActions(edge: .trailing) {
                    Button(model.deleteButtonTitle) {
                        model.onSelectNodeForDeletion(row)
                    }
                    .tint(Colors.red)
                }
            }
        case let .explorer(explorer):
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
        default:
            EmptyView()
        }
    }
}

// MARK: - Actions

extension ChainSettingsScene {
    private func onDeleteNode(_ row: GemNodeRow) {
        Task { await model.onDeleteNode(row) }
    }
}
