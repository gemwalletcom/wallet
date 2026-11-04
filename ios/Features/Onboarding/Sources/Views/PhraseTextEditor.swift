// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

@available(iOS 26.0, *)
struct PhraseTextEditor: View {
    @State private var model: PhraseTextEditorViewModel

    init(model: PhraseTextEditorViewModel) {
        _model = State(initialValue: model)
    }

    var body: some View {
        TextEditor(text: $model.content, selection: $model.selection)
            .attributedTextFormattingDefinition(PhraseAttributes.self)
            .writingToolsBehavior(.disabled)
            .scrollContentBackground(.hidden)
            .padding(.horizontal, -.space4)
            .padding(.vertical, -.space8)
            .overlay(alignment: .topLeading) {
                if model.showsPlaceholder {
                    Text(model.placeholder)
                        .foregroundStyle(Colors.gray)
                        .allowsHitTesting(false)
                }
            }
            .onChange(of: model.content, model.onChangeContent)
            .onChange(of: model.input.text, initial: true, model.onChangeText)
            .onChange(of: model.input.invalidWords, model.onChangeInvalidWords)
            .onChange(of: model.selection, model.onChangeSelection)
            .onChange(of: model.input.cursor, model.onChangeCursor)
    }
}

private struct PhraseAttributes: AttributeScope {
    let foregroundColor: AttributeScopes.SwiftUIAttributes.ForegroundColorAttribute
}
