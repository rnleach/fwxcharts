#
# This script assumes the following have already been set in the gnuplot environment.
# variables:
#   num_hours
#   now_time
#   start_time
#   end_time
#   main_title
#   output_name
#   output_prefix
#
# heredocs:
#   $data
#   $merged_data
#   $wet_dry_data

#
# Multiplot of plume stability data
#
reset
#
# Graph style
#
set output output_prefix."/".output_name

# palette
set palette defined (\
0 '#604860',\
1 '#784860',\
2 '#a86060',\
3 '#c07860',\
4 '#f0a848',\
5 '#f8ca8c',\
6 '#feecae',\
7 '#fff4c2',\
8 '#fff7db',\
9 '#fffcf6')
set cbrange [0:(num_hours)]
#
# Get the number of blocks in $data
#
stats $data using 1 nooutput # sets the STATS_blocks variable used below

#
# Set up x axis data
#
set xdata time
set timefmt "%Y-%m-%d-%H"
#
# Set up the multiplot
#
set multiplot layout 4,1 title main_title font ",14"
#
# Plot the top row, which is the HDW
#
set tmargin screen 0.95
set rmargin screen 0.85
set lmargin screen 0.1
set bmargin screen 0.75
set xtics scale 0
set format x ''
set ytics 100,100,700
set grid
unset colorbox
set arrow from now_time, graph 0 to now_time, graph 1 nohead lc rgb "black"
plot [start_time:end_time][0:700] $data u 1:5:2 w l lc palette t "HDW"
#
# Plot the middle row which is the energies
#
set tmargin screen 0.75
set rmargin screen 0.85
set lmargin screen 0.1
set bmargin screen 0.55
set xtics scale 0
set ytics 500,1000,2500
set grid
set arrow from now_time, graph 0 to now_time, graph 1 nohead lc rgb "black"
plot [start_time:end_time][0:3000] $data u 1:3:2 w l lc palette t "E0"
#
# Plot the bottom row which is the ratio
#
set tmargin screen 0.55
set rmargin screen 0.85
set lmargin screen 0.1
set bmargin screen 0.35
set grid
set logscale y
set ytics (0.1, 0.5, 1.5, 5, 10, 25)
set arrow from now_time, graph 0 to now_time, graph 1 nohead lc rgb "black"
plot [start_time:end_time][0.01:50] $data u 1:($4/$3):2 w l lc palette     t "dE/E0",\
				                    0.5                 w l lc rgb "black" notitle,\
					                1.5                 w l lc rgb "black"  notitle
#
# Make a heat map
#
set tmargin screen 0.35
set rmargin screen 0.85
set lmargin screen 0.1
set bmargin screen 0.15

# Set up x-axis
set xtics nomirror scale 1
set format x "%m/%d %H"
set format y "%2.0f"
unset logscale y
set xtics rotate by -45 offset 0, screen -0.035
set ytics 0,2,20
set xlabel "Date and hour [UTC]\n" font ",14" offset 0, screen -0.045
set view map
# Set up color map and color bar 
set palette defined (\
   0 '#ffffff',\
 250 '#ffffff',\
 250 '#ffff00',\
 500 '#ff0000',\
1000 '#00ffff',\
2000 '#ff00ff',\
3000 '#000000')
set cbrange [0:3000]
set colorbox vertical
splot [start_time:end_time][0:15] \
		  $wet_dry_data using 1:2:($3+$4) w pm3d           notitle, \
		  $merged_data  using 1:3:(0)     w lines lc black notitle
#
# Clean up
#
unset multiplot
