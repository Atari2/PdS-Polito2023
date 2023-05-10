Un noto rompicapo prevede di trovare la sequenza di operazioni elementari (somma, 
sottrazione, moltiplicazione e divisione) necessarie per ottenere 10, partendo da 5 numeri 
casuali da 0 a 9. I vincoli sono:
● le cinque cifre sono comprese tra 0 e 9 e possono ripetersi
● le cinque cifre devono essere utilizzate tutte, in qualsiasi ordine
● non ci sono limiti sulle operazioni (es. vanno bene anche quattro somme)
● non si considera la precedenza degli operatori, le operazioni vanno applicate da 
sinistra a destra secondo il loro ordine
Esempio: dato 2 7 2 2 1 una soluzione può essere 7 - 2 - 1 x 2 + 2 = 10 
Scrivere un programma che, letta la sequenza di numeri da command line come argomento, 
trovi tutte le possibili soluzioni, se ve ne sono, le salvi in un vettore di stringhe (es: “7 - 2 - 1 
x 2 + 2” ) e lo stampi.
In un primo momento utilizzare un approccio brute force, ovvero elencare tutte le possibili 
permutazioni delle cinque cifre e, per ogni permutazione, tutte le possibili permutazioni delle 
quattro operazioni, provando a calcolare il risultato per ciascuna di esse. Se il risultato è 10 
la permutazione viene salvata, altrimenti viene scartata.
Una volta verificato il corretto funzionamento, sfruttare dei thread per velocizzare la ricerca 
delle soluzioni in parallelo. Si divide la lista di tutte le possibili permutazioni in n blocchi 
uguali e per ciascuna si lancia un thread. Provare con n=2,3,4… ecc, misurare i tempi e 
trovare il numero di thread oltre il quale non vi sono vantaggi.
Cambia qualcosa se la divisione del lavoro fra thread anziché essere a blocchi è 
interleaved? Vale a dire con tre thread il primo prova le permutazioni con indice 0,3,6,... il 
secondo 1,4,7,... e il terzo 2,5,8,…