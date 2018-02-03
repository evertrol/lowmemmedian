#include <stdlib.h>
#include <stdio.h>
#include <math.h>
#include <float.h>
#include <stdint.h>
#include <stdbool.h>
#include <time.h>
#include <errno.h>
#include <pthread.h>


struct counter {
		double *data;
		size_t ndata;
		double partition;
		size_t nlow;
		size_t nhigh;
		double below;
		double above;
};

int calccounts_threads(double *data, size_t ndata, double partition,
			   size_t *nlow, size_t *nhigh, double *below, double *above);
void *calccounts_threaded(void *counts);
double calcgen(double *data, size_t ndata,
			   double maxdiff, double factor, double decrease);
double calc(double *data, size_t ndata);
int runtests(void);


void *
calccounts_threaded(void *counts)
{
		struct counter *lcounts = counts;

		lcounts->below = -(double)INFINITY;
		lcounts->above = (double)INFINITY;
		lcounts->nlow = 0;
		lcounts->nhigh = 0;

		for (size_t i = 0; i < lcounts->ndata; i++) {
				double value = lcounts->data[i];
				double partition = lcounts->partition;

				if (value <= partition) {
						lcounts->nlow += 1;
				}
				if (value >= partition) {
						lcounts->nhigh += 1;
				}
				if (value < partition && lcounts->below < value) {
						lcounts->below = value;
				}
				if (value > partition && lcounts->above > value) {
						lcounts->above = value;
				}
		}
		return NULL;
}


int
calccounts_threads(double *data, size_t ndata, double partition,
					size_t *nlow, size_t *nhigh, double *below, double *above)
{
		*below = -(double)INFINITY;
		*above = (double)INFINITY;
		*nlow = 0;
		*nhigh = 0;

		size_t nthreads = 2;
		char *nthreads_str = getenv("MEDIAN_NTHREADS");
		if (nthreads_str) {
				char *endptr;
				errno = 0;
				nthreads = (size_t)strtol(nthreads_str, &endptr, 10);
				if (errno || *endptr != '\0') {
						perror("invalid MEDIAN_NTHREADS value");
						nthreads = 2;
				}
		}

		errno = 0;
		pthread_t *threads = calloc(nthreads, sizeof *threads);
		struct counter *counts = calloc(nthreads, sizeof *counts);
		if (!threads || !counts) {
				perror("memory failure");
				exit(EXIT_FAILURE);
		}
		size_t step = ndata / nthreads;
		for (size_t i = 0; i < nthreads; i++) {
				counts[i].data = data + i*step;
				counts[i].ndata = step;
				counts[i].partition = partition;
				if (i == nthreads-1) {
						counts[i].ndata = ndata - i*step;
				}
				errno = 0;
				int status = pthread_create(
						&threads[i], NULL, calccounts_threaded, &counts[i]);
				if (status) {
						perror("error creating thread");
						exit(EXIT_FAILURE);
				}

		}


		for (size_t i = 0; i < nthreads; i++) {
				errno = 0;
				int status = pthread_join(threads[i], NULL);
				if (status) {
						perror("error joining thread");
						exit(EXIT_FAILURE);
				}
				*nlow += counts[i].nlow;
				*nhigh += counts[i].nhigh;
				if (*below < counts[i].below) {
						*below = counts[i].below;
				}
				if (*above > counts[i].above) {
						*above = counts[i].above;
				}
		}

		free(threads);
		free(counts);

		return 0;
}


double
calcgen(double *data, size_t ndata,
		double maxdiff, double factor, double decrease)
{
		if (ndata == 0) {
				return (double)NAN;
		}
		else if (ndata == 1) {
				return data[0];
		}
		else if (ndata == 2) {
				return (data[0] + data[1])/2.0;
		}

		double fact = factor;
		double prevdiff = (double)INFINITY;
		double sum = 0;
		for (size_t i = 0; i < ndata; i++) {
				sum += data[i];
		}
		double partition = sum / ndata;
		double prevpartition = partition;
		double delta = 0.0;
		double min = partition;
		double max = partition;
		for (size_t i = 0; i < ndata; i++) {
				double value = data[i];
				if (value < min) {
						min = value;
				} else if (value > max) {
						max = value;
				}
		}
		bool evenlen = !(ndata % 2);
		double below = -(double)INFINITY;
		double above = (double)INFINITY;
		size_t nlow = 0;
		size_t nhigh = 0;
		while (1) {
				calccounts_threads(data, ndata, partition, &nlow, &nhigh, &below, &above);
				size_t nsame = nhigh + nlow - ndata;

				if (nlow == nhigh) {
						if (nsame == 0) {
								partition = (below + above) / 2.0;
						}
						break;
				}
				else if (nlow > nhigh) {
						if (nlow - nhigh <= nsame) {
								if (nsame > 0) {
										if (evenlen && nsame == 1) {
												partition = (below + partition) / 2.0;
										}
								} else {
										if (evenlen) {
												partition = (below + above) / 2.0;
										} else {
												partition = below;
										}
								};
								break;
						}

						double diff = (double)(nlow - nhigh - nsame);
						if (diff > maxdiff) {
								if (fabs(prevdiff) < diff) {
										// The change was overestimated
										// Try again with a smaller scaling factor
										fact *= decrease;
										partition = prevpartition + prevdiff * fact * delta;
								} else {
										// Reset the scaling factor
										fact = factor;
										prevdiff = -diff;
										delta = above - below;
										prevpartition = partition;
										partition -= diff * fact * delta;
								}
						} else {
								partition = below;
						}
				} else {  // nlow < nhigh
						if (nhigh - nlow <= nsame) {
								if (nsame > 0) {
										if (evenlen && nsame == 1) {
												partition = (partition + above) / 2.0;
										}
								} else {
										if (evenlen) {
												partition = (below + above) / 2.0;
										} else {
												partition = above;
										}
								};
								break;
						}

						double diff = (double)(nhigh - nlow - nsame);
						if (diff > maxdiff) {
								if (fabs(prevdiff) < diff) {
										// The change was overestimated
										// Try again with a smaller scaling factor
										fact *= decrease;
										partition = prevpartition + prevdiff * fact * delta;
								} else {
										// Reset the scaling factor
										fact = factor;
										prevdiff = diff;
										delta = above - below;
										prevpartition = partition;
										partition += diff * fact  * delta;
								}
						} else {
								partition = above;
						}
				}
		}

		return partition;
}


double
calc(double *data, size_t ndata)
{
		return calcgen(data, ndata, 5.0, 0.2, 0.5);
}


int runtests(void)
{
        double data[] = {1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0};
		double m = calc(data, 7);
        if (fabs(m - 4.0) > 2 * DBL_EPSILON) {
				printf("%f != 4\n", m);
				return 1;
		}
		/*
        let data: Vec<f64> = vec![1.0, 1.0, 1.0, 4.0, 5.0, 6.0, 1.0];
        assert_eq!(1.0, calc(&data));
        let data: Vec<f64> = vec![1.0, 1.0, 2.0, 4.0, 5.0, 6.0, 1.0];
        assert_eq!(2.0, calc(&data));
        let data: Vec<f64> = vec![4.0, 2.0, 1.0, 7.0, 3.0, 6.0, 5.0];
        assert_eq!(4.0, calc(&data));
        let data: Vec<f64> = vec![7.0, 7.0, 1.0, 1.0, 5.0, 4.0, 3.0];
        assert_eq!(4.0, calc(&data));
        let data: Vec<f64> = vec![5.0, 3.0, 4.0, 7.0, 1.0, 6.0, 2.0];
        assert_eq!(4.0, calc(&data));
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![3.0, 5.0, 4.0, 8.0, 1.0, 7.0, 2.0, 6.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![4.0, 6.0, 3.0, 8.0, 1.0, 7.0, 2.0, 5.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![5.0, 6.0, 3.0, 8.0, 1.0, 7.0, 2.0, 4.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0];
        assert!((calc(&data) - 3.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0];
        assert!((calc(&data) - 3.0).abs() <= 2.0 * std::f64::EPSILON);


        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0];
        assert!((calc(&data) - 3.0).abs() <= 2.0 * std::f64::EPSILON);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![3.0, 5.0, 4.0, 8.0, 1.0, 7.0, 2.0, 6.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 3.0, 4.0];
        assert!((calc(&data) - 3.0).abs() <= 2.0 * std::f64::EPSILON);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 1.0, 1.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 1.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 1.0, 2.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 2.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![4.0, 2.0, 1.0, 7.0, 3.0, 6.0, 5.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![7.0, 7.0, 1.0, 1.0, 5.0, 4.0, 3.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![5.0, 3.0, 4.0, 7.0, 1.0, 6.0, 2.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 1.0, 1.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 1.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 1.0, 2.0, 4.0, 5.0, 6.0, 1.0];
        assert!((calc(&data) - 2.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![4.0, 2.0, 1.0, 7.0, 3.0, 6.0, 5.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![7.0, 7.0, 1.0, 1.0, 5.0, 4.0, 3.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![5.0, 3.0, 4.0, 7.0, 1.0, 6.0, 2.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 4.0, 5.0, 6.0, 7.0];
        assert!((calc(&data) - 4.0).abs() <= 2.0 * std::f64::EPSILON);

        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 6.0, 20.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);
        let data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 6.0, 6.0];
        assert!((calc(&data) - 4.5).abs() <= 2.0 * std::f64::EPSILON);

		*/
		return 0;
}


int
main(int argc, char *argv[]) {

		if (argc < 3) {
				printf("Running unit tests\n");
				int failed = runtests();
				if (failed) {
						return EXIT_FAILURE;
				}
				return EXIT_SUCCESS;

		}

		char *endptr;
		errno = 0;
		long nmax = strtol(argv[1], &endptr, 10);
		if (errno || *endptr != '\0') {
				fprintf(stderr, "n is not an integer\n");
				return EXIT_FAILURE;
		}
		if (nmax <= 0) {
				fprintf(stderr, "n should be larger than 0\n");
				return EXIT_FAILURE;
		}
		size_t ndata = (size_t)nmax;

		errno = 0;
		double *data = malloc(ndata * sizeof *data);
		if (!data) {
				fprintf(stderr, "Memory allocation failed\n");
				return EXIT_FAILURE;
		}

		errno = 0;
		FILE *fp = fopen(argv[2], "r");
		if (!fp) {
				fprintf(stderr, "Could not open file %s\n", argv[2]);
				return EXIT_FAILURE;
		}

		for (size_t i = 0; i < ndata; i++) {
				int n = fscanf(fp, "%lf", &data[i]);
				if (n != 1) {
						fprintf(stderr, "Failed to read value in file %s\n",
								argv[2]);
						return EXIT_FAILURE;
				}
		}
		fclose(fp);
		printf("%zu\n", ndata);
		clock_t start = clock(), diff;
		double median = calc(data, ndata);
		diff = clock() - start;
		double sec = (double)diff / CLOCKS_PER_SEC;
		printf("Median = %.15lf (%f)\n", median, sec);

		free(data);

		return EXIT_SUCCESS;
}
