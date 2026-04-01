import SwiftUI

public extension View {
    func numericTransition(for value: some Equatable) -> some View {
        numericTransition(for: [value])
    }

    func numericTransition(for values: [some Equatable]) -> some View {
        contentTransition(.numericText(countsDown: true))
            .animation(.default, value: values)
    }
}
