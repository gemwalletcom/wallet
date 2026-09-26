// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemAutocloseFieldState
import GemstonePrimitives
import Localization
import PrimitivesComponents
import Style
import SwiftUI

public struct AutocloseInputSection<Field: Hashable>: View {
    @Binding var text: String
    let state: GemAutocloseFieldState
    let field: Field
    var focusedField: FocusState<Field?>.Binding

    public init(
        text: Binding<String>,
        state: GemAutocloseFieldState,
        field: Field,
        focusedField: FocusState<Field?>.Binding,
    ) {
        _text = text
        self.state = state
        self.field = field
        self.focusedField = focusedField
    }

    public var body: some View {
        Section {
            FloatTextField(Localized.Asset.price, text: $text, allowClean: true)
                .keyboardType(.decimalPad)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()
                .focused(focusedField, equals: field)

            if let message = state.validation.errorDescription {
                Text(.init(message))
                    .textStyle(TextStyle(font: .footnote, color: Colors.red))
                    .transition(.opacity)
            }
        } header: {
            Text(state.tpslType.toPrimitives().autocloseTitle)
        } footer: {
            HStack {
                Text(state.estimateTitle.text)
                Spacer()
                Text(state.estimate?.text ?? Placeholder.empty)
                    .foregroundStyle(state.tone.color)
            }
            .font(.subheadline)
            .fontWeight(.semibold)
        }
    }
}
