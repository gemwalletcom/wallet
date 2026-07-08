// Copyright (c). Gem Wallet. All rights reserved.

import Style
import SwiftUI

public struct SuffixTextField<Field: Hashable>: View {
    private let suffix: String
    private let sanitizer: ((String) -> String)?
    @Binding private var text: String
    private let field: Field
    private var focusedField: FocusState<Field?>.Binding

    public init(
        suffix: String,
        sanitizer: ((String) -> String)? = nil,
        text: Binding<String>,
        field: Field,
        focusedField: FocusState<Field?>.Binding,
    ) {
        self.suffix = suffix
        self.sanitizer = sanitizer
        _text = text
        self.field = field
        self.focusedField = focusedField
    }

    public var body: some View {
        HStack(spacing: .zero) {
            TextField("", text: $text)
                .keyboardType(.decimalPad)
                .multilineTextAlignment(.trailing)
                .focused(focusedField, equals: field)
            Text(suffix)
        }
        .foregroundStyle(Colors.gray)
        .onChange(of: text) { _, newValue in
            guard let sanitizer else { return }
            let sanitized = sanitizer(newValue)
            if sanitized != newValue {
                text = sanitized
            }
        }
    }
}
