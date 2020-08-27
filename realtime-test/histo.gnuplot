# Plot with e.g. gnuplot -p -e "filename='compube-1.csv'" histo.gnuplot

binwidth=5
bin(x,width)=width*floor(x/width)

plot filename using (bin($1,binwidth)):(1.0) smooth freq with boxes
