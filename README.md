# KIFS Ray Marching

## Opis
Namen projekta je ustavriti simulacijo fraktalnih struktur poznanih pod imenom "Kaleidoscopic Iterated Function Systems", oziroma KIFS na krajše.
Originalna ideja izvira iz objave na [fractalforums.com](http://www.fractalforums.com/ifs-iterated-function-systems/kaleidoscopic-(escape-time-ifs)/), vendar ta spletna stran ni več aktivna.
Lahko pa objavo najdete na spletni strani [Wayback Machine](https://web.archive.org/) z vnosom URL naslova foruma v iskalnik.

Izhajamo iz koncepta "Distance Estimated Fractals", kjer iz funkcije, ki nam vrača približke razdalj do fraktala, izrišemo želeno množico.
To posplošimo z dodajanjem rotacij, prepogibov, raztegov in translacij ter tako tvorimo nove, bolj zanimive fraktale.

Risanje na podlagi funkcije razdalje pa poteka po metodi "Ray Marching".
Iz točke opazovanja se sprehajamo po žarkih, kjer se na vsakem koraku premaknemo za razdaljo do fraktala.
V določenem številu korakov se potem fraktalu dovolj približamo, da lahko točko izrišemo, ali pa ga povsem zgrešimo.

To bo na koncu predstavljeno v aplikaciji, kjer bo lahko uporabnik ročno generiral različne fraktalne strukture glede na vhodne podatke, oziroma jih bo program naključno tvoril sam.
