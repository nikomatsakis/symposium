// Keep the diagram visible inside its anchor even when the evidence is collapsed.
for (const figure of document.querySelectorAll("figure[data-anchor]")) {
    const anchor = figure.previousElementSibling;
    if (anchor?.classList.contains("ab-anchor") && anchor.id === figure.dataset.anchor) {
        anchor.append(figure);
    }
}

const bookTheme = document.documentElement.classList;
const darkDiagram = () => ["ayu", "coal", "navy"].some(theme => bookTheme.contains(theme));
const initialDark = darkDiagram();
mermaid.initialize({ startOnLoad: false, theme: initialDark ? "dark" : "default" });

async function renderDiagrams() {
    const linksByDiagram = new Map();
    for (const diagram of document.querySelectorAll(".chapter-diagram .mermaid")) {
        const links = new Map();
        // Let Mermaid lay out the visible message text before adding its SVG link.
        diagram.textContent = diagram.textContent.replace(
            /^([ \t]*\w+[-]+>>\w+:\s*)\[([^\]\n]+)\]\(([\w-]+\.html#[\w-]+)\)$/gm,
            (_, arrow, label, href) => {
                links.set(label, href);
                return arrow + label;
            }
        );
        linksByDiagram.set(diagram, links);
    }
    await mermaid.run();
    for (const [diagram, links] of linksByDiagram) {
        for (const label of diagram.querySelectorAll(".messageText")) {
            const href = links.get(label.textContent.trim());
            if (!href) continue;
            const link = document.createElementNS(label.namespaceURI, "a");
            link.setAttribute("href", href);
            link.classList.add("message-link");
            label.replaceWith(link);
            link.append(label);
        }
    }
}

if (document.querySelector(".mermaid")) {
    renderDiagrams();
    new MutationObserver(() => {
        if (darkDiagram() !== initialDark) location.reload();
    }).observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
}
