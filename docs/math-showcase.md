# Math rendering showcase

This file is a manual and automated QA fixture. It is intentionally not loaded
at startup.

Inline flow keeps prose on the same line: $E = mc^2$, $\frac{a}{b}$,
$\sqrt{x^2+y^2}$, and Unicode text $\text{速度 } v$ continue into ordinary
words after each formula.

Nested Markdown keeps its own styling around math: **energy $E=mc^2$**, a
[linked formula $a^2+b^2=c^2$](https://example.com), and `literal $source$`.

$$
\int_0^\infty e^{-x^2}\,dx = \frac{\sqrt{\pi}}{2}
$$

$$
\begin{cases}
x^2 & x > 0 \\
0   & x \le 0
\end{cases}
$$

```math
\begin{matrix}
a & b \\
c & d
\end{matrix}
```

The following expression is deliberately wide and should scroll horizontally
instead of being clipped:

$$
\left(\frac{a_1+b_1+c_1+d_1+e_1+f_1+g_1+h_1+i_1+j_1+k_1+l_1}{a_2+b_2+c_2+d_2+e_2+f_2+g_2+h_2+i_2+j_2+k_2+l_2}\right)^{\sum_{n=1}^{100} n}
$$

Invalid source stays visible and editable:

$$
\frac{unclosed}{formula
$$

Visual Edit QA: click the leading and trailing halves of each valid formula,
drag a selection across it, then move focus away. Only the focused formula's
complete `$...$`, `$$...$$`, or fenced source should be revealed.
